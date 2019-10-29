#version 450

layout(push_constant) uniform PCData {
    int multiple;
} pc;

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;

void main() {
    uint idx = gl_GlobalInvocationID.x;

    buf.data[idx] = idx * pc.multiple;
}
