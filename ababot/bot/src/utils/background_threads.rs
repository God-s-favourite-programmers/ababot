use std::sync::Arc;

use serenity::prelude::TypeMapKey;

pub struct ThreadStorage {
    pub running: bool,
}

impl TypeMapKey for ThreadStorage {
    type Value = Arc<ThreadStorage>;
}
