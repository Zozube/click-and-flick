use bevy::prelude::*;

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    println!("Screen despawned");
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}
