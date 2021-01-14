#version 460
#extension GL_NV_ray_tracing : require
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_GOOGLE_include_directive : enable

#include "../triangle_data.h"

#define G_UNIFORM_SET 0
#include "../global_uniforms.h"
#undef G_UNIFORM_SET

#include "../payload.h"
layout(location = 0) rayPayloadInNV InitialPayload PAYLOAD_GENERAL;

hitAttributeNV vec3 attribs;

void main() {
    // FIXME: maybe we should avoid deref multiple times and store the struct?? 
    const mat3 transform = mat3(
        BLAS_TRIANGLE_DATA[gl_InstanceID].data[gl_PrimitiveID].tangeant,
        BLAS_TRIANGLE_DATA[gl_InstanceID].data[gl_PrimitiveID].bitangeant,
        BLAS_TRIANGLE_DATA[gl_InstanceID].data[gl_PrimitiveID].normal
    );

    const vec3 orig =  BLAS_TRIANGLE_DATA[gl_InstanceID].data[gl_PrimitiveID].tex_orig
                    +  BLAS_TRIANGLE_DATA[gl_InstanceID].data[gl_PrimitiveID].tex_u * attribs.x
                    +  BLAS_TRIANGLE_DATA[gl_InstanceID].data[gl_PrimitiveID].tex_v * attribs.y;

    const float lod = gl_RayTmaxNV / 10.0;

    // normal deformation
    const vec3 normal = transform * (2 * textureLod(UNI_TEXTURE_ARRAY, orig + vec3(0, 0, 1), lod).xyz - vec3(1.0));
    const vec3 illum = max(dot(-UNI_SCENE.sunDirection, normal), 0.0) * textureLod(UNI_TEXTURE_ARRAY, orig, lod).xyz;
    const vec3 coeffs = textureLod(UNI_TEXTURE_ARRAY, orig + vec3(0, 0, 2), lod).xyz;

    PAYLOAD_GENERAL.hit = true;
    PAYLOAD_GENERAL.normal = normal;
    PAYLOAD_GENERAL.distance = gl_RayTmaxNV;
    PAYLOAD_GENERAL.hit_position = gl_WorldRayOriginNV + gl_WorldRayDirectionNV * gl_HitTNV;
    PAYLOAD_GENERAL.illumination = illum;
    PAYLOAD_GENERAL.mer = coeffs;
}
