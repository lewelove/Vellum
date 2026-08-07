pub mod config;
#[cfg(test)]
mod tests;

use anyhow::{Context, Result};
use config::{ActionConfig, AppConfig, CoversConfig, CoversRegistry, InterfaceConfig};
use indexmap::IndexMap;
use mlua::serde::SerializeOptions;
use mlua::{Lua, LuaSerdeExt, Table};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const LUA_CORE: &str = include_str!("core.lua");
const LUA_UTILS_FN: &str = include_str!("utils/fn.lua");
const LUA_UTILS_FS: &str = include_str!("utils/fs.lua");
const LUA_CONFIG: &str = include_str!("config.lua");
const LUA_COMPILER: &str = include_str!("compiler.lua");
const LUA_ACTIONS: &str = include_str!("actions.lua");
const LUA_LOGIC: &str = include_str!("logic.lua");

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct FilterDef {
    pub label: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct GrouperDef {
    pub label: String,
    #[serde(default)]
    pub strict: bool,
    #[serde(default)]
    pub index: Option<bool>,
    #[serde(default)]
    pub count: Option<bool>,
    #[serde(default)]
    pub reverse: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct OrderDef {
    pub label: String,
    #[serde(default)]
    pub reverse: bool,
    #[serde(default)]
    pub strict: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct LibraryDef {
    pub label: String,
    #[serde(default)]
    pub strict: bool,
    #[serde(default)]
    pub filters: Vec<String>,
    #[serde(default)]
    pub groupers: Vec<String>,
    #[serde(default)]
    pub orders: Vec<String>,
    #[serde(skip_deserializing, default)]
    pub allowed_filters: Vec<String>,
    #[serde(skip_deserializing, default)]
    pub allowed_groupers: Vec<String>,
    #[serde(skip_deserializing, default)]
    pub allowed_orders: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct ShelfDef {
    pub label: String,
    #[serde(default)]
    pub reverse: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct LogicManifest {
    pub filters: IndexMap<String, FilterDef>,
    pub groupers: IndexMap<String, GrouperDef>,
    pub orders: IndexMap<String, OrderDef>,
    pub libraries: IndexMap<String, LibraryDef>,
    pub shelves: IndexMap<String, ShelfDef>,
    #[serde(default)]
    pub filters_order: Vec<String>,
    #[serde(default)]
    pub groupers_order: Vec<String>,
    #[serde(default)]
    pub orders_order: Vec<String>,
    #[serde(default)]
    pub libraries_order: Vec<String>,
    #[serde(default)]
    pub shelves_order: Vec<String>,
}

impl LogicManifest {
    pub fn normalize(&mut self) {
        self.filters_order = self.filters.keys().cloned().collect();
        self.groupers_order = self.groupers.keys().cloned().collect();
        self.orders_order = self.orders.keys().cloned().collect();
        self.libraries_order = self.libraries.keys().cloned().collect();
        self.shelves_order = self.shelves.keys().cloned().collect();

        for (_, g) in &mut self.groupers {
            let idx = g.index.unwrap_or(false);
            g.index = Some(idx);
            if g.count.is_none() {
                g.count = Some(!idx);
            }
        }

        let global_groupers: HashSet<String> = self
            .groupers
            .iter()
            .filter(|(_, g)| !g.strict)
            .map(|(id, _)| id.clone())
            .collect();

        let global_orders: HashSet<String> = self
            .orders
            .iter()
            .filter(|(_, s)| !s.strict)
            .map(|(id, _)| id.clone())
            .collect();

        for (_, library) in &mut self.libraries {
            let mut allowed_filter_ids = Vec::new();
            for f in &library.filters {
                if self.filters.contains_key(f) {
                    allowed_filter_ids.push(f.clone());
                }
            }
            library.allowed_filters = allowed_filter_ids;

            let mut allowed_grouper_ids = HashSet::new();
            for g in &library.groupers {
                allowed_grouper_ids.insert(g.clone());
            }
            if !library.strict {
                for g in &global_groupers {
                    allowed_grouper_ids.insert(g.clone());
                }
            }

            let mut allowed_order_ids = HashSet::new();
            for s in &library.orders {
                allowed_order_ids.insert(s.clone());
            }
            if !library.strict {
                for s in &global_orders {
                    allowed_order_ids.insert(s.clone());
                }
            }

            library.allowed_groupers = self
                .groupers
                .keys()
                .filter(|k| allowed_grouper_ids.contains(*k))
                .cloned()
                .collect();

            library.allowed_orders = self
                .orders
                .keys()
                .filter(|k| allowed_order_ids.contains(*k))
                .cloned()
                .collect();
        }
    }
}

pub struct LuaEngine {
    pub lua: Lua,
}

pub struct EvaluatedLuaData {
    pub app: AppConfig,
    pub covers: CoversRegistry,
    pub interfaces: HashMap<String, InterfaceConfig>,
    pub actions: HashMap<String, ActionConfig>,
    pub dependencies: Vec<PathBuf>,
    pub manifest: LogicManifest,
}

fn register_native_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let vl: mlua::Table = globals.get("vl").unwrap_or_else(|_| lua.create_table().unwrap());

    let opts = SerializeOptions::new()
        .serialize_none_to_null(false)
        .serialize_unit_to_null(false);

    let json_table = lua.create_table().map_err(|e| anyhow::anyhow!("{e}"))?;
    json_table.set(
        "decode",
        lua.create_function(move |lua, s: String| {
            let val: serde_json::Value = serde_json::from_str(&s)
                .map_err(mlua::Error::external)?;
            lua.to_value_with(&val, opts)
        }).map_err(|e| anyhow::anyhow!("{e}"))?,
    ).map_err(|e| anyhow::anyhow!("{e}"))?;

    json_table.set(
        "encode",
        lua.create_function(|lua, val: mlua::Value| {
            let json_val: serde_json::Value = lua.from_value(val)?;
            serde_json::to_string(&json_val).map_err(mlua::Error::external)
        }).map_err(|e| anyhow::anyhow!("{e}"))?,
    ).map_err(|e| anyhow::anyhow!("{e}"))?;

    vl.set("json", json_table).map_err(|e| anyhow::anyhow!("{e}"))?;

    let toml_table = lua.create_table().map_err(|e| anyhow::anyhow!("{e}"))?;
    toml_table.set(
        "decode",
        lua.create_function(move |lua, s: String| {
            let toml_val: toml::Value = toml::from_str(&s)
                .map_err(mlua::Error::external)?;
            let json_val = crate::types::toml_to_json(toml_val);
            lua.to_value_with(&json_val, opts)
        }).map_err(|e| anyhow::anyhow!("{e}"))?,
    ).map_err(|e| anyhow::anyhow!("{e}"))?;

    toml_table.set(
        "encode",
        lua.create_function(|lua, val: mlua::Value| {
            let json_val: serde_json::Value = lua.from_value(val)?;
            let toml_val = crate::types::json_to_toml(json_val);
            toml::to_string_pretty(&toml_val).map_err(mlua::Error::external)
        }).map_err(|e| anyhow::anyhow!("{e}"))?,
    ).map_err(|e| anyhow::anyhow!("{e}"))?;

    vl.set("toml", toml_table).map_err(|e| anyhow::anyhow!("{e}"))?;

    globals.set("vl", vl).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}

impl LuaEngine {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        register_native_functions(&lua)?;
        lua.load(LUA_CORE)
            .exec()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to load core.lua")?;
        lua.load(LUA_UTILS_FN)
            .exec()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to load utils/fn.lua")?;
        lua.load(LUA_UTILS_FS)
            .exec()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to load utils/fs.lua")?;
        lua.load(LUA_CONFIG)
            .exec()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to load config.lua")?;
        lua.load(LUA_COMPILER)
            .exec()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to load compiler.lua")?;
        lua.load(LUA_ACTIONS)
            .exec()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to load actions.lua")?;
        lua.load(LUA_LOGIC)
            .exec()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to load logic.lua")?;
        Ok(Self { lua })
    }

    pub fn evaluate_config(&self, path: &Path) -> Result<EvaluatedLuaData> {
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read {}", path.display()))?;

        if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy();
            let code = format!(
                "package.path = package.path .. ';{}/?.lua'",
                parent_str.replace('\\', "/")
            );
            let _: () = self
                .lua
                .load(&code)
                .exec()
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("Failed to append config directory to package.path")?;
        }

        self.lua
            .load(&content)
            .set_name(path.to_string_lossy())
            .exec()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to execute vellum.lua")?;

        let globals = self.lua.globals();

        let get_config: mlua::Function = globals
            .get("__VELLUM_GET_CONFIG")
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let config_table: Table = get_config.call(()).map_err(|e| anyhow::anyhow!("{e}"))?;
        let app_config: AppConfig = self
            .lua
            .from_value(mlua::Value::Table(config_table))
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let get_covers: mlua::Function = globals
            .get("__VELLUM_GET_COVERS")
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let covers_table: Table = get_covers.call(()).map_err(|e| anyhow::anyhow!("{e}"))?;
        let covers: CoversRegistry = self
            .lua
            .from_value(mlua::Value::Table(covers_table))
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let get_interfaces: mlua::Function = globals
            .get("__VELLUM_GET_INTERFACES")
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let interfaces_table: Table =
            get_interfaces.call(()).map_err(|e| anyhow::anyhow!("{e}"))?;
        let interfaces: HashMap<String, InterfaceConfig> = self
            .lua
            .from_value(mlua::Value::Table(interfaces_table))
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let get_actions: mlua::Function = globals
            .get("__VELLUM_GET_ACTIONS")
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let actions_table: Table = get_actions.call(()).map_err(|e| anyhow::anyhow!("{e}"))?;
        let actions: HashMap<String, ActionConfig> = self
            .lua
            .from_value(mlua::Value::Table(actions_table))
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let get_deps: mlua::Function = globals
            .get("__VELLUM_GET_DEPENDENCIES")
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let deps_table: Table = get_deps.call(()).map_err(|e| anyhow::anyhow!("{e}"))?;
        let deps_str: Vec<String> = self
            .lua
            .from_value(mlua::Value::Table(deps_table))
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let get_manifest: mlua::Function = globals
            .get("__VELLUM_GET_LOGIC_MANIFEST")
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let manifest_table: Table = get_manifest.call(()).map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut manifest: LogicManifest = self
            .lua
            .from_value(mlua::Value::Table(manifest_table))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        manifest.normalize();

        let mut dependencies = Vec::new();
        for d in deps_str {
            let p = crate::utils::expand_path(&d);
            dependencies.push(p.canonicalize().unwrap_or(p));
        }

        Ok(EvaluatedLuaData {
            app: app_config,
            covers,
            interfaces,
            actions,
            dependencies,
            manifest,
        })
    }

    pub fn evaluate_album_logic(
        &self,
        album_val: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let globals = self.lua.globals();
        let eval_fn: mlua::Function = globals
            .get("__VELLUM_EVALUATE_ALBUM_LOGIC")
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let opts = SerializeOptions::new()
            .serialize_none_to_null(false)
            .serialize_unit_to_null(false);
        let lua_album = self
            .lua
            .to_value_with(album_val, opts)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let res: mlua::Table = eval_fn.call(lua_album).map_err(|e| anyhow::anyhow!("{e}"))?;
        let json_res: serde_json::Value = self
            .lua
            .from_value(mlua::Value::Table(res))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(json_res)
    }

    pub fn execute_dispatcher(
        &self,
        ctx_val: &serde_json::Value,
        manifests_val: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let globals = self.lua.globals();
        let dispatcher: mlua::Function = globals
            .get("__VELLUM_DISPATCHER")
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let opts = SerializeOptions::new()
            .serialize_none_to_null(false)
            .serialize_unit_to_null(false);

        let lua_ctx = self
            .lua
            .to_value_with(ctx_val, opts)
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let lua_manifests = self
            .lua
            .to_value_with(manifests_val, opts)
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let res: mlua::Table = dispatcher
            .call((lua_ctx, lua_manifests))
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let json_res: serde_json::Value = self
            .lua
            .from_value(mlua::Value::Table(res))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(json_res)
    }
}

pub fn resolve_config_path() -> Option<PathBuf> {
    if let Some(home_config) = dirs::home_dir().map(|h| h.join(".config/vellum/vellum.lua"))
        && home_config.exists()
    {
        return Some(home_config);
    }

    if let Ok(env_path) = std::env::var("VELLUM_CONFIG_PATH") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }

    let mut curr = std::env::current_dir().ok()?;
    loop {
        let local_nested = curr.join("config/vellum.lua");
        if local_nested.exists() {
            return Some(local_nested);
        }

        let local_root = curr.join("vellum.lua");
        if local_root.exists() {
            return Some(local_root);
        }

        if let Some(parent) = curr.parent() {
            curr = parent.to_path_buf();
        } else {
            break;
        }
    }

    None
}

#[derive(Clone, Debug)]
pub struct ResolvedConfig {
    pub app: AppConfig,
    pub covers: CoversRegistry,
    pub interfaces: HashMap<String, InterfaceConfig>,
    pub actions: HashMap<String, ActionConfig>,
    pub dependencies: Vec<PathBuf>,
    pub manifest: LogicManifest,
    pub path: PathBuf,
}

impl ResolvedConfig {
    pub fn load() -> Result<Self> {
        let path = resolve_config_path().context("Could not locate vellum.lua")?;
        let engine = LuaEngine::new()?;
        let mut evaluated = engine.evaluate_config(&path)?;

        if evaluated.covers.targets.is_empty() {
            evaluated.covers.targets.push(CoversConfig {
                filter: "lanczos".to_string(),
                size: 200,
            });
        }

        let mut dependencies = evaluated.dependencies;
        dependencies.push(path.clone().canonicalize().unwrap_or_else(|_| path.clone()));

        Ok(Self {
            app: evaluated.app,
            covers: evaluated.covers,
            interfaces: evaluated.interfaces,
            actions: evaluated.actions,
            dependencies,
            manifest: evaluated.manifest,
            path,
        })
    }
}

thread_local! {
    pub static LUA_VM: RefCell<Option<LuaEngine>> = const { RefCell::new(None) };
}

pub fn reset_lua_vm() {
    LUA_VM.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

pub fn get_or_init_lua_vm<F, R>(config_path: &Path, f: F) -> Result<R>
where
    F: FnOnce(&LuaEngine) -> Result<R>,
{
    LUA_VM.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_none() {
            let engine = LuaEngine::new()?;
            engine.evaluate_config(config_path)?;
            *opt = Some(engine);
        }
        f(opt.as_ref().unwrap())
    })
}
