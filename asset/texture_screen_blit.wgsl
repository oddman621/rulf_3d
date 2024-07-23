struct SurfaceInfo {
	width: u32,
	height: u32
}

struct Rect {
	left: u32, right: u32, top: u32, bottom: u32
}

@group(0) @binding(0) var<uniform> surface: SurfaceInfo;
@group(0) @binding(1) var<uniform> rect: Rect;

@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var smp: sampler;

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) uv: vec2<f32>,
	@location(1) depth: f32
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
	let half_surface = SurfaceInfo(
		surface.width / 2u, surface.height / 2u
	);
	let rect_norm = vec4<f32>(
		f32(rect.left) / f32(half_surface.width) - 1.0,
		f32(rect.right) / f32(half_surface.width) - 1.0,
		-f32(rect.top) / f32(half_surface.height) + 1.0,
		-f32(rect.bottom) / f32(half_surface.height) + 1.0,
	);
	
	var pos: vec2<f32>;
	var uv: vec2<f32>;
	
	switch idx {
		// 0: right bottom
		// 1: right top
		// 2: left bottom
		// 3: left top
		case 0u {
			pos = rect_norm.yw;
			uv = vec2<f32>(1.0, 1.0);
		}
		case 1u {
			pos = rect_norm.yz;
			uv = vec2<f32>(1.0, 0.0);
		}
		case 2u {
			pos = rect_norm.xw;
			uv = vec2<f32>(0.0, 1.0);
		}
		case 3u {
			pos = rect_norm.xz;
			uv = vec2<f32>(0.0, 0.0);
		}
		default {
			//ERROR
			//NOTE: Is there a way to discard in vertex shader?
		}
	}

	var out: VertexOutput;
	out.clip_position = vec4<f32>(pos, 1.0, 1.0);
	out.uv = uv;
	out.depth = 0.0; //TODO: Accept depth(distance).
	return out;
}

struct FragmentOutput {
	@location(0) color: vec4<f32>,
	@builtin(frag_depth) depth: f32
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
	var out: FragmentOutput;
	out.color = textureSample(tex, smp, in.uv);
	out.depth = in.depth;
	return out;
}