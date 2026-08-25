#[cfg(test)]
mod tests;

use mlua::{Lua, Table, Value};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

type ReaderHandle = JoinHandle<Vec<u8>>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnvMode {
    Inherit,
    Clear,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StreamMode {
    Capture,
    Ignore,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExecutionMode {
    Sync,
    Detach,
}

struct SystemOpts {
    cwd: Option<String>,
    env: Option<HashMap<String, String>>,
    clear_env: EnvMode,
    stdin: Option<String>,
    stdout: StreamMode,
    stderr: StreamMode,
    timeout: Option<u64>,
    mode: ExecutionMode,
}

fn parse_opts(opts_val: Option<Table>) -> mlua::Result<SystemOpts> {
    if let Some(tbl) = opts_val {
        let is_detach = tbl.get::<Option<bool>>("detach")?.unwrap_or(false);
        let default_stdio = !is_detach;
        let clear_env = if tbl.get::<Option<bool>>("clear_env")?.unwrap_or(false) {
            EnvMode::Clear
        } else {
            EnvMode::Inherit
        };
        let stdout = if tbl.get::<Option<bool>>("stdout")?.unwrap_or(default_stdio) {
            StreamMode::Capture
        } else {
            StreamMode::Ignore
        };
        let stderr = if tbl.get::<Option<bool>>("stderr")?.unwrap_or(default_stdio) {
            StreamMode::Capture
        } else {
            StreamMode::Ignore
        };
        let mode = if is_detach {
            ExecutionMode::Detach
        } else {
            ExecutionMode::Sync
        };
        Ok(SystemOpts {
            cwd: tbl.get("cwd")?,
            env: tbl.get("env")?,
            clear_env,
            stdin: tbl.get("stdin")?,
            stdout,
            stderr,
            timeout: tbl.get("timeout")?,
            mode,
        })
    } else {
        Ok(SystemOpts {
            cwd: None,
            env: None,
            clear_env: EnvMode::Inherit,
            stdin: None,
            stdout: StreamMode::Capture,
            stderr: StreamMode::Capture,
            timeout: None,
            mode: ExecutionMode::Sync,
        })
    }
}

fn parse_cmd(cmd_val: Value) -> mlua::Result<Vec<String>> {
    match cmd_val {
        Value::Table(tbl) => {
            let mut args = Vec::new();
            for item in tbl.sequence_values::<String>() {
                args.push(item?);
            }
            if args.is_empty() || args[0].trim().is_empty() {
                return Err(mlua::Error::runtime("cmd table cannot be empty"));
            }
            Ok(args)
        }
        Value::String(s) => {
            let cmd = s.to_str()?.to_string();
            if cmd.trim().is_empty() {
                return Err(mlua::Error::runtime("cmd string cannot be empty"));
            }
            Ok(vec!["sh".to_string(), "-c".to_string(), cmd])
        }
        _ => Err(mlua::Error::runtime(
            "cmd must be a table of strings or a string",
        )),
    }
}

fn build_command(cmd_args: &[String], opts: &SystemOpts) -> Command {
    let mut command = Command::new(&cmd_args[0]);
    if cmd_args.len() > 1 {
        command.args(&cmd_args[1..]);
    }

    if opts.clear_env == EnvMode::Clear {
        command.env_clear();
    }
    if let Some(ref env_map) = opts.env {
        for (k, v) in env_map {
            command.env(k, v);
        }
    }
    if let Some(ref cwd) = opts.cwd {
        command.current_dir(crate::utils::expand_path(cwd));
    }
    command
}

fn spawn_detached(
    mut command: Command,
    opts: &SystemOpts,
    lua: &Lua,
) -> mlua::Result<Value> {
    command.stdin(Stdio::null());
    command.stdout(if opts.stdout == StreamMode::Capture {
        Stdio::inherit()
    } else {
        Stdio::null()
    });
    command.stderr(if opts.stderr == StreamMode::Capture {
        Stdio::inherit()
    } else {
        Stdio::null()
    });
    command.process_group(0);

    let mut child = command.spawn().map_err(mlua::Error::external)?;
    let pid = child.id();

    thread::spawn(move || {
        let _ = child.wait();
    });

    let ret = lua.create_table()?;
    ret.set("pid", pid)?;
    ret.set("ok", true)?;
    Ok(Value::Table(ret))
}

fn spawn_stream_readers(
    child: &mut Child,
) -> (Option<ReaderHandle>, Option<ReaderHandle>) {
    let stdout_handle = child.stdout.take().map(|mut r| {
        thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = r.read_to_end(&mut buf);
            buf
        })
    });

    let stderr_handle = child.stderr.take().map(|mut r| {
        thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = r.read_to_end(&mut buf);
            buf
        })
    });

    (stdout_handle, stderr_handle)
}

fn build_result_table(
    status: ExitStatus,
    stdout_bytes: &[u8],
    stderr_bytes: &[u8],
    lua: &Lua,
) -> mlua::Result<Value> {
    let code_val = status
        .code()
        .map_or(Value::Nil, |c| Value::Integer(i64::from(c)));
    let signal_val = status
        .signal()
        .map_or(Value::Nil, |s| Value::Integer(i64::from(s)));

    let ret = lua.create_table()?;
    ret.set("code", code_val)?;
    ret.set("signal", signal_val)?;
    ret.set("stdout", String::from_utf8_lossy(stdout_bytes).into_owned())?;
    ret.set("stderr", String::from_utf8_lossy(stderr_bytes).into_owned())?;
    ret.set("ok", status.success())?;

    Ok(Value::Table(ret))
}

fn wait_for_exit(
    mut child: Child,
    timeout_dur: Option<Duration>,
    stdout_handle: Option<ReaderHandle>,
    stderr_handle: Option<ReaderHandle>,
    lua: &Lua,
) -> mlua::Result<Value> {
    let status = match timeout_dur {
        None => child.wait().map_err(mlua::Error::external)?,
        Some(dur) => {
            let start = Instant::now();
            loop {
                if let Some(status) = child.try_wait().map_err(mlua::Error::external)? {
                    break status;
                }
                if start.elapsed() >= dur {
                    let _ = child.kill();
                    let status = child.wait().map_err(mlua::Error::external)?;
                    let stdout_bytes = stdout_handle
                        .map_or_else(Vec::new, |h| h.join().unwrap_or_default());
                    let stderr_bytes = stderr_handle
                        .map_or_else(Vec::new, |h| h.join().unwrap_or_default());
                    let timeout_ms = dur.as_millis();
                    let stderr_msg = String::from_utf8_lossy(&stderr_bytes);

                    let code_val = status
                        .code()
                        .map_or(Value::Nil, |c| Value::Integer(i64::from(c)));
                    let signal_val = status
                        .signal()
                        .map_or(Value::Integer(9), |s| Value::Integer(i64::from(s)));

                    let ret = lua.create_table()?;
                    ret.set("code", code_val)?;
                    ret.set("signal", signal_val)?;
                    ret.set(
                        "stdout",
                        String::from_utf8_lossy(&stdout_bytes).into_owned(),
                    )?;
                    ret.set(
                        "stderr",
                        format!("Process timed out after {timeout_ms}ms\n{stderr_msg}"),
                    )?;
                    ret.set("ok", false)?;
                    return Ok(Value::Table(ret));
                }
                thread::sleep(Duration::from_millis(1));
            }
        }
    };

    let stdout_bytes =
        stdout_handle.map_or_else(Vec::new, |h| h.join().unwrap_or_default());
    let stderr_bytes =
        stderr_handle.map_or_else(Vec::new, |h| h.join().unwrap_or_default());

    build_result_table(status, &stdout_bytes, &stderr_bytes, lua)
}

fn execute_sync(
    mut command: Command,
    opts: SystemOpts,
    lua: &Lua,
) -> mlua::Result<Value> {
    command.stdin(if opts.stdin.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    command.stdout(if opts.stdout == StreamMode::Capture {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    command.stderr(if opts.stderr == StreamMode::Capture {
        Stdio::piped()
    } else {
        Stdio::null()
    });

    let mut child = command.spawn().map_err(mlua::Error::external)?;
    let (stdout_handle, stderr_handle) = spawn_stream_readers(&mut child);

    let stdin_handle = if let Some(stdin_str) = opts.stdin
        && let Some(mut child_stdin) = child.stdin.take()
    {
        Some(thread::spawn(move || {
            let _ = child_stdin.write_all(stdin_str.as_bytes());
        }))
    } else {
        None
    };

    let timeout_dur = opts.timeout.map(Duration::from_millis);
    let result = wait_for_exit(child, timeout_dur, stdout_handle, stderr_handle, lua);

    if let Some(h) = stdin_handle {
        let _ = h.join();
    }

    result
}

pub fn lua_system(
    lua: &Lua,
    (cmd_val, opts_val): (Value, Option<Table>),
) -> mlua::Result<Value> {
    let cmd_args = parse_cmd(cmd_val)?;
    let opts = parse_opts(opts_val)?;
    let command = build_command(&cmd_args, &opts);

    if opts.mode == ExecutionMode::Detach {
        spawn_detached(command, &opts, lua)
    } else {
        execute_sync(command, opts, lua)
    }
}

pub fn register(lua: &Lua, dale_tbl: &Table) -> mlua::Result<()> {
    dale_tbl.set("system", lua.create_function(lua_system)?)
}
