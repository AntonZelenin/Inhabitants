use crate::planet::resources::PlanetGenerationSettings;
use planetgen::generator::PlanetGenerator;
use planetgen::planet::PlanetData;

pub fn generate_planet_data(settings: &PlanetGenerationSettings) -> PlanetData {
    planetgen::reload_config();
    let generator = configure_planet_generator(settings);
    generator.generate()
}

/// Pure business logic: Configure planet generator from settings
fn configure_planet_generator(settings: &PlanetGenerationSettings) -> PlanetGenerator {
    let mut generator = PlanetGenerator::new(settings.radius);
    generator.num_plates = settings.num_plates;
    generator.num_micro_plates = settings.num_micro_plates;
    generator.seed = settings.seed;
    generator.flow_warp_freq = settings.flow_warp_freq;
    generator.flow_warp_steps = settings.flow_warp_steps;
    generator.flow_warp_step_angle = settings.flow_warp_step_angle;
    generator.wind_speed = settings.wind_speed;

    // Apply custom continent configuration from UI settings
    let continent_config = planetgen::config::ContinentConfig {
        continent_frequency: settings.continent_frequency,
        continent_amplitude: settings.continent_amplitude,
        distortion_frequency: settings.distortion_frequency,
        distortion_amplitude: settings.distortion_amplitude,
        detail_frequency: settings.detail_frequency,
        detail_amplitude: settings.detail_amplitude,
        continent_threshold: settings.continent_threshold,
        ocean_depth_amplitude: settings.ocean_depth_amplitude,
    };
    generator.with_continent_config(continent_config);

    // Apply mountain configuration from UI settings
    generator.mountain_height = settings.mountain_height;
    generator.mountain_width = settings.mountain_width;

    generator
}
