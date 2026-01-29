use bevy::prelude::*;

/// Marker component for the wind texture entity
/// This marks the entity that holds the wind field texture (cube-sphere atlas)
#[derive(Component)]
pub struct WindMapMarker;

/// Component that holds the wind field texture handle
#[derive(Component)]
pub struct WindTextureHandle(pub Handle<Image>);
