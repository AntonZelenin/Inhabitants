use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

pub struct InternalAudioPlugin;

// This plugin is responsible to control the game audio
impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin);
    }
}

#[derive(Resource)]
struct FlyingAudio(Handle<AudioInstance>);

// example of audio usage
// fn start_audio(mut commands: Commands, audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
//     audio.pause();
//     let handle = audio
//         .play(audio_assets.flying.clone())
//         .looped()
//         .with_volume(0.3)
//         .handle();
//     commands.insert_resource(FlyingAudio(handle));
// }
