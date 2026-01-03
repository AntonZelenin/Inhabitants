# Continent Generation System

## Overview

The continent generation system creates realistic planetary terrain using layered Perlin noise functions. It uses a multi-stage approach inspired by procedural planet generation techniques, adapted for spherical cube-mapped surfaces.

## Architecture

### Two Systems Available

1. **Simple System** (`ContinentNoiseConfig`)
   - Basic two-layer noise (continent + detail)
   - Fast and straightforward
   - Good for prototyping

2. **Advanced System** (`AdvancedContinentNoise`)
   - Multi-layered noise with terrain type selection
   - More realistic continent shapes
   - Better coastline variation and continental shelves

## Advanced System Design

### Stage 1: Base Continent Definition

**Purpose**: Define where continents exist and their rough shape.

**Components**:
- **Base FBM (Fractional Brownian Motion)**
  - High octave count (14) for detail at all zoom levels
  - Frequency controlled by `continent_frequency`
  - Lacunarity ~2.2 for natural-looking variation
  
- **Curve Module**
  - Reshapes the FBM output to create realistic continent profiles
  - Creates gentle slopes near sea level
  - Defines where high elevations can occur
  
- **Carver Module**
  - Higher-frequency FBM (4.34x base frequency)
  - Creates valleys, passes, and breaks in terrain
  - Prevents continents from being solid blocks

**Output**: Base continent elevation map (-1.0 to +1.0)

### Stage 2: Terrain Type Definition

**Purpose**: Determine what kind of terrain appears at each location (plains, hills, or rougher areas).

**Components**:
- **Terrain Type FBM**
  - Lower frequency for large terrain zones
  - Creates natural boundaries between terrain types
  
- **Terrain Offset**
  - Shifts terrain types based on elevation
  - Low values: rough terrain only at peaks
  - High values: rough terrain at any elevation

**Output**: Terrain type selector value

### Stage 3: Terrain Layers

Each terrain type generates its own elevation contribution:

#### Hills Terrain
- **Billow noise** for rolling, rounded hills
- Moderate frequency and amplitude
- Creates gentle undulations across continents

#### Plains Terrain
- **Low-amplitude Billow noise**
- Minimal vertical variation
- Represents flat continental interiors

### Stage 4: Terrain Type Selection

**Purpose**: Blend different terrain types based on elevation and the terrain selector.

**Process**:
1. Use terrain type FBM to create selection boundaries
2. Apply `Select` modules to choose between terrain types
3. Blend smoothly at boundaries using falloff ranges

**Selection Thresholds**:
- Plains: Below threshold
- Hills: Between plains and rough terrain thresholds
- (Mountains excluded for now - reserved for tectonic simulation)

### Stage 7: Terrain Scaling

**Purpose**: Scale each terrain type to appropriate elevation ranges.

**Scaling Factors**:
- **Hills**: Medium scale (creates noticeable elevation changes)
- **Plains**: Minimal scale (keeps terrain relatively flat)
- **Continent base**: Scaled by `CONTINENT_HEIGHT_SCALE`

### Stage 8: Continental Shelf & Ocean

**Purpose**: Create realistic ocean depth variation and continental shelf transitions.

**Components**:
- **Continental Shelf**
  - Terraces near coastlines (using `Terrace` module)
  - Creates shallow shelf regions before deep ocean
  - Threshold at `SHELF_LEVEL` (typically -0.375)

- **Ocean Trenches**
  - Scaled depth variation controlled by `ocean_depth_amplitude`
  - Prevents perfectly flat ocean floors
  - Creates deep trenches and shallower regions

### Stage 9: Final Assembly

**Purpose**: Combine all layers into final elevation values.

**Process**:
1. Start with base continent elevation
2. Add scaled plains terrain where appropriate
3. Add scaled hills terrain in hilly regions
4. Apply continental shelf in shallow ocean
5. Apply ocean depth variation in deep ocean
6. Clamp to valid elevation range

## Parameters

### User-Configurable (in UI)

- **continent_frequency** (0.5 - 3.0)
  - Base frequency of continent noise
  - Lower = larger continents
  - Higher = smaller, more numerous continents

- **continent_amplitude** (0.1 - 2.0)
  - Height scaling for continental landmasses
  - Controls maximum elevation difference

- **continent_threshold** (-0.5 - 0.5)
  - Sea level / land coverage
  - Lower = more land
  - Higher = more ocean

- **detail_frequency** (5.0 - 20.0)
  - Frequency of surface detail noise
  - Higher = more jagged coastlines and finer surface texture

- **detail_amplitude** (0.05 - 0.5)
  - Strength of surface detail
  - Controls how rough/smooth terrain appears

- **ocean_depth_amplitude** (0.1 - 2.0)
  - Scaling for ocean depth variation
  - Higher = deeper trenches and more variation

### Internal Constants

- **CONTINENT_LACUNARITY** (2.208984375)
  - Controls frequency increase between octaves
  - Affects continent shape complexity

- **HILLS_LACUNARITY** (2.162109375)
  - Similar to above, but for hills
  
- **SHELF_LEVEL** (-0.375)
  - Elevation where continental shelf appears
  - Must be below sea level

- **TERRAIN_OFFSET** (1.0)
  - Shifts terrain type boundaries
  - Affects where rough vs smooth terrain appears

- **CONTINENT_HEIGHT_SCALE** (calculated from sea level)
  - Base scaling for continent elevations
  - Automatically adjusts based on sea level

## Noise Functions Used

### FBM (Fractional Brownian Motion)
- Multiple octaves of Perlin noise
- Each octave adds finer detail
- Used for: base continents, terrain selection, carving

### Billow
- Similar to FBM but creates billowy, cloud-like patterns
- Absolute value makes it symmetrical
- Used for: hills and plains

### Curve
- Remaps input values through control points
- Creates custom response curves
- Used for: shaping continent profiles

### Terrace
- Creates stepped/terraced elevations
- Used for: continental shelf

### Select
- Chooses between two inputs based on a control signal
- Supports smooth blending (falloff)
- Used for: terrain type selection

## 3D Spherical Sampling

Unlike the flat plane example, this system samples noise in 3D space along sphere surface normals:

```rust
let dir = Vec3::from(cube_face_point(face_idx, u, v)).normalize();
let height = advanced_noise.sample_height(dir);
```

**Benefits**:
- Seamless continuity across cube faces
- No distortion from UV mapping
- Natural spherical terrain

## Future Enhancements

### Planned
- **Mountains**: Generated by tectonic plate simulation (not noise)
- **Rivers**: Carved based on elevation flow and precipitation
- **Biomes**: Temperature and moisture-based terrain variation
- **Erosion**: Post-processing to simulate weathering

### Possible
- **Badlands**: Layered sedimentary formations
- **Glaciers**: Ice coverage at high elevations/latitudes
- **Volcanic regions**: Hotspot-based terrain features
- **Impact craters**: Large-scale surface features

## Performance Considerations

- **Caching**: Results are cached in `heightmap` arrays
- **Face-based generation**: 6 cube faces generated independently
- **Parallel-ready**: Each face can be generated in parallel (future optimization)
- **Octave count**: Higher octaves = more detail but slower generation

## Debugging

To visualize individual noise layers:
1. Add debug output in `sample_height()`
2. Return intermediate values (base continent, terrain type, etc.)
3. Visualize in planet mesh with color coding
4. Use UI sliders to isolate specific parameters

## References

- libnoise complex planet example
- Perlin noise (Ken Perlin, 1983)
- Fractional Brownian Motion for terrain generation
- Cube mapping for spherical surfaces

