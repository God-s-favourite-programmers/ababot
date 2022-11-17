@group(0) @binding(0) var<storage, read> a: array<u32>;
@group(0) @binding(1) var<storage, read_write> b: array<u32>;


@compute @workgroup_size(32)
fn main(@builtin(global_invocation_id) idx: vec3<u32>) {
    let i = idx.x;
    b[i] = a[i] * 2u;
}
