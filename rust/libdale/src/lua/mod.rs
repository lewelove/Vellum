#[cfg(test)]
mod tests;
pub mod utils;

use crate::config::{ActionConfig, AppConfig, CoversConfig, CoversRegistry, InterfaceConfig};
use anyhow::{Context, Result};
use indexmap::IndexMap;
use mlua::serde::SerializeOptions;
use mlua::{Lua, LuaSerdeExt, Table};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const LUA_CORE: &str = include_str!("core.lua");
const LUA_UTILS_FN: &str = include_str!("utils/fn.lua");
const LUA_CONFIG: &str = include_str!("config.lua");
const LUA_COMPILER: &str = include_str!("compiler.lua");
const LUA_ACTIONS: &str = include_str!("actions.lua");
const LUA_LOGIC: &str = include_str!("logic.lua");

#[derive(Clone, Debug)]
pub struct EngineContext {
    pub cache_root: PathBuf,
    pub captured_deps: Arc<std::sync::Mutex<HashSet<PathBuf>>>,
}

impl EngineContext {
    pub fn new(cache_root: PathBuf) -> Self {
        Self {
            cache_root,
            captured_deps: Arc::new(std::sync::Mutex::new(HashSet::new())),
        }
    }

    pub fn record_dependency(&self, path: PathBuf) {
        if let Ok(mut deps) = self.captured_deps.lock() {
            let canon = path.canonicalize().unwrap_or(path);
            deps.insert(canon);
        }
    }

    pub fn take_dependencies(&self) -> HashSet<PathBuf> {
        self.captured_deps
            .lock()
            .map(|mut deps| std::mem::take(&mut *deps))
            .unwrap_or_default()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct FilterDef {
    pub label: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct GrouperDef {
    pub label: String,
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
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct LibraryDef {
    pub label: String,
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
pub struct CabinetDef {
    pub label: String,
    #[serde(default)]
    pub shelves: Vec<String>,
    #[serde(default)]
    pub orders: Vec<String>,
    #[serde(skip_deserializing, default)]
    pub allowed_shelves: Vec<String>,
    #[serde(skip_deserializing, default)]
    pub allowed_orders: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct LogicManifest {
    pub filters: IndexMap<String, FilterDef>,
    pub groupers: IndexMap<String, GrouperDef>,
    pub orders: IndexMap<String, OrderDef>,
    pub libraries: IndexMap<String, LibraryDef>,
    pub shelves: IndexMap<String, ShelfDef>,
    pub cabinets: IndexMap<String, CabinetDef>,
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
    #[serde(default)]
    pub cabinets_order: Vec<String>,
}

impl LogicManifest {
    pub fn normalize(&mut self) {
        if !self.filters_order.is_empty() {
            self.filters.sort_by_cached_key(|k, _| {
                self.filters_order.iter().position(|x| x == k).unwrap_or(usize::MAX)
            });
        } else {
            self.filters_order = self.filters.keys().cloned().collect();
        }

        if !self.groupers_order.is_empty() {
            self.groupers.sort_by_cached_key(|k, _| {
                self.groupers_order.iter().position(|x| x == k).unwrap_or(usize::MAX)
            });
        } else {
            self.groupers_order = self.groupers.keys().cloned().collect();
        }

        if !self.orders_order.is_empty() {
            self.orders.sort_by_cached_key(|k, _| {
                self.orders_order.iter().position(|x| x == k).unwrap_or(usize::MAX)
            });
        } else {
            self.orders_order = self.orders.keys().cloned().collect();
        }

        if !self.libraries_order.is_empty() {
            self.libraries.sort_by_cached_key(|k, _| {
                self.libraries_order.iter().position(|x| x == k).unwrap_or(usize::MAX)
            });
        } else {
            self.libraries_order = self.libraries.keys().cloned().collect();
        }

        if !self.shelves_order.is_empty() {
            self.shelves.sort_by_cached_key(|k, _| {
                self.shelves_order.iter().position(|x| x == k).unwrap_or(usize::MAX)
            });
        } else {
            self.shelves_order = self.shelves.keys().cloned().collect();
        }

        if !self.cabinets_order.is_empty() {
            self.cabinets.sort_by_cached_key(|k, _| {
                self.cabinets_order.iter().position(|x| x == k).unwrap_or(usize::MAX)
            });
        } else {
            self.cabinets_order = self.cabinets.keys().cloned().collect();
        }

        for (_, g) in &mut self.groupers {
            let idx = g.index.unwrap_or(false);
            g.index = Some(idx);
            if g.count.is_none() {
                g.count = Some(!idx);
            }
        }

        for (_, library) in &mut self.libraries {
            library.allowed_filters = library
                .filters
                .iter()
                .filter(|f| self.filters.contains_key(*f))
                .cloned()
                .collect();

            library.allowed_groupers = library
                .groupers
                .iter()
                .filter(|g| self.groupers.contains_key(*g))
                .cloned()
                .collect();

            library.allowed_orders = library
                .orders
                .iter()
                .filter(|o| self.orders.contains_key(*o))
                .cloned()
                .collect();
        }

        for (_, cabinet) in &mut self.cabinets {
            cabinet.allowed_shelves = cabinet
                .shelves
                .iter()
                .filter(|s| self.shelves.contains_key(*s))
                .cloned()
                .collect();

            cabinet.allowed_orders = cabinet
                .orders
                .iter()
                .filter(|o| self.orders.contains_key(*o))
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

fn create_json_table(lua: &Lua, opts: SerializeOptions) -> mlua::Result<Table> {
    let json_table = lua.create_table()?;
    json_table.set(
        "decode",
        lua.create_function(move |lua, s: String| {
            let val: serde_json::Value = serde_json::from_str(&s)
                .map_err(mlua::Error::external)?;
            lua.to_value_with(&val, opts)
        })?,
    )?;

    json_table.set(
        "encode",
        lua.create_function(|lua, val: mlua::Value| {
            let json_val: serde_json::Value = lua.from_value(val)?;
            serde_json::to_string(&json_val).map_err(mlua::Error::external)
        })?,
    )?;

    Ok(json_table)
}

fn create_toml_table(lua: &Lua, opts: SerializeOptions) -> mlua::Result<Table> {
    let toml_table = lua.create_table()?;
    toml_table.set(
        "decode",
        lua.create_function(move |lua, s: String| {
            let toml_val: toml::Value = toml::from_str(&s)
                .map_err(mlua::Error::external)?;
            let json_val = crate::types::toml_to_json(toml_val);
            lua.to_value_with(&json_val, opts)
        })?,
    )?;

    toml_table.set(
        "encode",
        lua.create_function(|lua, val: mlua::Value| {
            let json_val: serde_json::Value = lua.from_value(val)?;
            let toml_val = crate::types::json_to_toml(json_val);
            toml::to_string_pretty(&toml_val).map_err(mlua::Error::external)
        })?,
    )?;

    Ok(toml_table)
}

fn call_getter<T: serde::de::DeserializeOwned>(lua: &Lua, fn_name: &str) -> Result<T> {
    let func: mlua::Function = lua
        .globals()
        .get(fn_name)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let val: mlua::Value = func.call(()).map_err(|e| anyhow::anyhow!("{e}"))?;
    let parsed: T = lua.from_value(val).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(parsed)
}

fn register_native_functions(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();
    let dale_table: Table = globals
        .get::<Table>("dale")
        .or_else(|_| lua.create_table())?;

    let opts = SerializeOptions::new()
        .serialize_none_to_null(false)
        .serialize_unit_to_null(false);

    dale_table.set("json", create_json_table(lua, opts)?)?;
    dale_table.set("toml", create_toml_table(lua, opts)?)?;
    dale_table.set("fs", utils::fs::create_fs_table(lua, opts)?)?;

    globals.set("dale", dale_table.clone())?;
    globals.set("d", dale_table)?;
    Ok(())
}

impl LuaEngine {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        let default_cache = crate::utils::expand_path("~/.cache/dale");
        lua.set_app_data(EngineContext::new(default_cache));

        register_native_functions(&lua)
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to register native Lua functions")?;
        lua.load(LUA_CORE)
            .exec()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to load core.lua")?;
        lua.load(LUA_UTILS_FN)
            .exec()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to load utils/fn.lua")?;
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

            if let Ok(registry) = self.lua.globals().get::<Table>("REGISTRY") {
                let _ = registry.set("config_dir", parent_str.to_string());
            }
        }

        self.lua
            .load(&content)
            .set_name(path.to_string_lossy())
            .exec()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to execute init.lua")?;

        let app_config: AppConfig = call_getter(&self.lua, "__DALE_GET_CONFIG")?;

        let config_dir = path.parent().unwrap_or_else(|| Path::new("."));
        let resolved_cache = if app_config.storage.cache.is_empty() {
            crate::utils::expand_path("~/.cache/dale")
        } else {
            crate::utils::resolve_path(&app_config.storage.cache, config_dir)
        };
        self.lua.set_app_data(EngineContext::new(resolved_cache));

        let covers: CoversRegistry = call_getter(&self.lua, "__DALE_GET_COVERS")?;
        let interfaces: HashMap<String, InterfaceConfig> =
            call_getter(&self.lua, "__DALE_GET_INTERFACES")?;
        let actions: HashMap<String, ActionConfig> = call_getter(&self.lua, "__DALE_GET_ACTIONS")?;
        let deps_str: Vec<String> = call_getter(&self.lua, "__DALE_GET_DEPENDENCIES")?;
        let mut manifest: LogicManifest = call_getter(&self.lua, "__DALE_GET_LOGIC_MANIFEST")?;
        manifest.normalize();

        let mut dependencies = Vec::new();
        for d in deps_str {
            let p = crate::utils::expand_path(&d);
            dependencies.push(p.canonicalize().unwrap_or(p));
        }

        if let Some(ctx) = self.lua.app_data_ref::<EngineContext>() {
            for dep in ctx.take_dependencies() {
                dependencies.push(dep);
            }
        }
        dependencies.sort();
        dependencies.dedup();

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
            .get("__DALE_EVALUATE_ALBUM_LOGIC")
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
            .get("__DALE_DISPATCHER")
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
    if let Some(home_config) = dirs::home_dir().map(|h| h.join(".config/dale/init.lua"))
        && home_config.exists()
    {
        return Some(home_config);
    }

    if let Ok(env_path) = std::env::var("DALE_CONFIG_PATH") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }

    let mut curr = std::env::current_dir().ok()?;
    loop {
        let local_nested = curr.join("config/init.lua");
        if local_nested.exists() {
            return Some(local_nested);
        }

        let local_root = curr.join("init.lua");
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
        let path = resolve_config_path().context("Could not locate init.lua")?;
        let config_dir = path.parent().unwrap_or_else(|| Path::new("."));
        let engine = LuaEngine::new()?;
        let mut evaluated = engine.evaluate_config(&path)?;

        if let Some(ref manifests) = evaluated.app.compiler.manifests {
            let validated = crate::compiler::manifest::validate_and_filter_manifest_names(manifests)?;
            evaluated.app.compiler.manifests = Some(validated);
        }

        if evaluated.covers.targets.is_empty() {
            evaluated.covers.targets.push(CoversConfig {
                filter: "lanczos".to_string(),
                size: 200,
            });
        }

        if !evaluated.app.storage.music_directory.is_empty() {
            evaluated.app.storage.music_directory = crate::utils::resolve_path(&evaluated.app.storage.music_directory, config_dir).to_string_lossy().to_string();
        }
        if let Some(ref env_path) = evaluated.app.storage.environment {
            evaluated.app.storage.environment = Some(crate::utils::resolve_path(env_path, config_dir).to_string_lossy().to_string());
        }
        if !evaluated.app.storage.cache.is_empty() {
            evaluated.app.storage.cache = crate::utils::resolve_path(&evaluated.app.storage.cache, config_dir).to_string_lossy().to_string();
        }
        if !evaluated.app.storage.state.is_empty() {
            evaluated.app.storage.state = crate::utils::resolve_path(&evaluated.app.storage.state, config_dir).to_string_lossy().to_string();
        }

        for intf in evaluated.interfaces.values_mut() {
            if let Some(ref dir) = intf.directory {
                intf.directory = Some(crate::utils::resolve_path(dir, config_dir).to_string_lossy().to_string());
            }
            if let Some(ref run_script) = intf.run {
                intf.run = Some(crate::utils::resolve_path(run_script, config_dir).to_string_lossy().to_string());
            }
            for asset_path in intf.assets.values_mut() {
                *asset_path = crate::utils::resolve_path(asset_path, config_dir).to_string_lossy().to_string();
            }
        }

        for action in evaluated.actions.values_mut() {
            if let Some(ref run_script) = action.run {
                action.run = Some(crate::utils::resolve_path(run_script, config_dir).to_string_lossy().to_string());
            }
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

pub fn with_evaluated_lua_vm<F, R>(config_path: &Path, f: F) -> Result<R>
where
    F: FnOnce(&LuaEngine, EvaluatedLuaData) -> Result<R>,
{
    let engine = LuaEngine::new()?;
    let eval = engine.evaluate_config(config_path)?;
    f(&engine, eval)
}
