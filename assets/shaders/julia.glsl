#version 110

uniform sampler1D colors;
uniform vec2 seed;
uniform int iterations;

varying vec2 v_Texture;

void main() {
	vec2 z;
	int  n;

	z.x = 3.0 * (v_Texture.x - 0.5);
	z.y = 2.0 * (v_Texture.y - 0.5);

	for (n = 0; n < iterations; n++) {
		float x = (z.x * z.x - z.y * z.y) + seed.x;
		float y = (z.y * z.x + z.x * z.y) + seed.y;

		if ((x * x + y * y) > 4.0) {
			break;
		}

		z.x = x;
		z.y = y;
	}

	gl_FragColor = texture1D(colors, (n == iterations ? 0.0 : float(n)) / 100.0);
}
