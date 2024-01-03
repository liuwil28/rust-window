struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texture_coords: vec2<f32>,
    @location(2) normal: vec3<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>
}

struct InstanceInput {
    @location(3) transform_matrix_0: vec4<f32>,
    @location(4) transform_matrix_1: vec4<f32>,
    @location(5) transform_matrix_2: vec4<f32>,
    @location(6) transform_matrix_3: vec4<f32>,
    @location(7) normal_matrix_0: vec3<f32>,
    @location(8) normal_matrix_1: vec3<f32>,
    @location(9) normal_matrix_2: vec3<f32>
}

struct Camera {
    position: vec4<f32>,
    proj_matrix: mat4x4<f32>
}

struct Light {
    position: vec3<f32>,
    color: vec3<f32>
}

@group(1) @binding(0)
var<uniform> camera: Camera;

@group(2) @binding(0)
var<uniform> light: Light;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let transform_matrix = mat4x4<f32>(
        instance.transform_matrix_0,
        instance.transform_matrix_1,
        instance.transform_matrix_2,
        instance.transform_matrix_3
    );

    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2
    );

    var out: VertexOutput;
    out.texture_coords = vertex.texture_coords;
    out.world_normal = normal_matrix * vertex.normal;
    var world_position: vec4<f32> = transform_matrix * vec4<f32>(vertex.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.proj_matrix * world_position;
    return out;
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let object_color = textureSample(texture, texture_sampler, vertex.texture_coords);

    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;
    
    let light_dir = normalize(light.position - vertex.world_position);
    
    let diffuse_strength = max(dot(vertex.world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let view_dir = normalize(camera.position.xyz - vertex.world_position);
    let reflect_dir = reflect(-light_dir, vertex.world_normal); // Phong
    // let half_dir = normalize(view_dir + light_dir); // Blinn-Phong

    // let specular_strength = pow(max(dot(view_dir, vertex.world_normal), 0.0), 32.0);
    let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0); // Phong
    // let specular_strength = pow(max(dot(vertex.world_normal, half_dir), 0.0), 32.0); // Blinn-Phong
    let specular_color = specular_strength * light.color;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;

    return vec4<f32>(result, object_color.a);
    // return vec4<f32>(specular_color, object_color.a);
}
