#version 110

uniform sampler1D colors;
uniform vec2 center;
uniform float scale;
uniform int iterations;

varying vec2 v_Texture;

void main() {
	vec2 z;
	vec2 c;
	int  n;

	c.x = 1.3333 * (v_Texture.x - 0.5) * scale - center.x;
	c.y = (v_Texture.y - 0.5) * scale - center.y;

	z = c;
	for (n = 0; n < iterations; n++) {
		float x = (z.x * z.x - z.y * z.y) + c.x;
		float y = (z.y * z.x + z.x * z.y) + c.y;

		if ((x * x + y * y) > 4.0) {
			break;
		}

		z.x = x;
		z.y = y;
	}

	gl_FragColor = texture1D(colors, (n == iterations ? 0.0 : float(n)) / 100.0);
}
