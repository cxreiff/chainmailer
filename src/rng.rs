use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Resource, Deref, DerefMut)]
pub struct RngResource(pub ChaCha8Rng);

impl Default for RngResource {
    fn default() -> Self {
        Self(ChaCha8Rng::from_os_rng())
    }
}
