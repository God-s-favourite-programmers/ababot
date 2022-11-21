@group(0) @binding(0)
var<storage, read> in_data: array<u32>;
@group(0) @binding(1)
var<storage, read_write> out_data: array<u32>;

@compute @workgroup_size(32)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var index: u32 = global_id.x;
    var value: u32 = in_data[index];
    for(var i = 0; i < 10; i++) {
        value *= 2u;
    }
    out_data[index] = value;
}