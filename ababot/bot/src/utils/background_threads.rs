use std::sync::Arc;

use serenity::prelude::TypeMapKey;

pub struct ThreadCounter {
    pub running: bool,
}

impl TypeMapKey for ThreadCounter {
    type Value = Arc<ThreadCounter>;
}
