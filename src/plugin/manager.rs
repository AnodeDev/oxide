use std::collections::HashMap;

use crate::plugin::Plugin;

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}
