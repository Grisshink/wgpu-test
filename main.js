const vertexSourceBack = `
    attribute vec2 posAttr;
    attribute vec2 uvAttr;

    varying mediump vec2 uv;

    void main() {
        gl_Position = vec4(posAttr, 0.0, 1.0);
        uv = uvAttr;
    }
`;
const fragmentSourceBack = `
    varying mediump vec2 uv;

    uniform mediump float time;
    uniform mediump vec3 bg_col;
    uniform mediump vec3 fg_col;
    uniform mediump float aspect;

    mediump float rand2D(mediump vec2 co) { 
        return fract(sin(dot(co.xy, vec2(12.9898,78.233))) * 43758.5453);
    }

    mediump float rand3D(mediump vec3 co) {
        return fract(sin(dot(co.xyz, vec3(12.9898,78.233,144.7272))) * 43758.5453);
    }

    mediump float perlin_layer(mediump vec3 co) {
        mediump vec3 quant_floor = floor(co);
        mediump vec3 quant_frac = fract(co);

        mediump float bot_left_back    = rand3D(vec3(quant_floor.x      , quant_floor.y      , quant_floor.z      ));
        mediump float bot_right_back   = rand3D(vec3(quant_floor.x + 1.0, quant_floor.y      , quant_floor.z      ));
        mediump float top_left_back    = rand3D(vec3(quant_floor.x      , quant_floor.y + 1.0, quant_floor.z      ));
        mediump float top_right_back   = rand3D(vec3(quant_floor.x + 1.0, quant_floor.y + 1.0, quant_floor.z      ));

        mediump float bot_left_front   = rand3D(vec3(quant_floor.x      , quant_floor.y      , quant_floor.z + 1.0));
        mediump float bot_right_front  = rand3D(vec3(quant_floor.x + 1.0, quant_floor.y      , quant_floor.z + 1.0));
        mediump float top_left_front   = rand3D(vec3(quant_floor.x      , quant_floor.y + 1.0, quant_floor.z + 1.0));
        mediump float top_right_front  = rand3D(vec3(quant_floor.x + 1.0, quant_floor.y + 1.0, quant_floor.z + 1.0));
        
        mediump float bot_back = mix(bot_left_back, bot_right_back, quant_frac.x);
        mediump float top_back = mix(top_left_back, top_right_back, quant_frac.x);

        mediump float bot_front = mix(bot_left_front, bot_right_front, quant_frac.x);
        mediump float top_front = mix(top_left_front, top_right_front, quant_frac.x);

        mediump float back = mix(bot_back, top_back, quant_frac.y);
        mediump float front = mix(bot_front, top_front, quant_frac.y);

        return mix(back, front, quant_frac.z);
    }

    mediump float perlin(mediump vec3 co) {
        mediump float out_val = 0.0;
        for (int i = 0; i < 8; i++) out_val += perlin_layer(co * pow(2.0, float(i)));
        return out_val / 8.0;
    }

    void main() {
        mediump vec2 uv_s = vec2(uv.x * aspect, uv.y) * 5.0;
        mediump float value = perlin(vec3(uv_s, time * 0.1));
        gl_FragColor = vec4(mix(bg_col, fg_col, value > 0.58 ? 1.0 : value > 0.53 ? 0.25 : 0.0), 1.0);
    }
`;

const vertexSourceText = `
    attribute vec2 posAttr;
    attribute vec2 uvAttr;

    varying mediump vec2 uv;

    void main() {
        gl_Position = vec4(posAttr, 0.0, 1.0);
        uv = uvAttr;
    }
`;
const fragmentSourceText = `
    varying mediump vec2 uv;

    uniform sampler2D text;

    void main() {
        mediump vec4 sample = texture2D(text, vec2(uv.x, 1.0 - uv.y));
        gl_FragColor = sample;
    }
`;

const vertexSourceFront = `
    attribute vec2 posAttr;
    attribute vec2 uvAttr;

    varying mediump vec2 uv;

    void main() {
        gl_Position = vec4(posAttr, 0.0, 1.0);
        uv = uvAttr;
    }
`;
const fragmentSourceFront = `
    varying mediump vec2 uv;

    uniform sampler2D back_buffer;
    uniform mediump float time;

    #define PI 3.141592653589793

    void main() {
        mediump vec2 uv_s = vec2(uv.x + cos(time * 0.45 + 2.0 * PI * uv.y) * 0.05, uv.y + sin(time * 0.6 + 2.0 * PI * uv.x) * 0.06);
        mediump vec2 uv_mod = abs(mod(uv_s + 1.0, 2.0) - 1.0);
        gl_FragColor = texture2D(back_buffer, uv_mod);
    }
`;

function loadShader(gl, type, source) {
    const shader = gl.createShader(type);

    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        console.error(`An error occurred compiling the shaders: ${gl.getShaderInfoLog(shader)}`);
        gl.deleteShader(shader);
        return null;
    }

    return shader;
}

function initShader(gl, vs, fs, attribs) {
    const vertexShader = loadShader(gl, gl.VERTEX_SHADER, vs);
    if (!vertexShader) return null;
    const fragmentShader = loadShader(gl, gl.FRAGMENT_SHADER, fs);
    if (!fragmentShader) return null;

    const shaderProgram = gl.createProgram();
    gl.attachShader(shaderProgram, vertexShader);
    gl.attachShader(shaderProgram, fragmentShader);

    for (let attrib of attribs) {
        gl.bindAttribLocation(shaderProgram, attrib.location, attrib.name);
    }

    gl.linkProgram(shaderProgram);

    if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
        console.error(`Unable to initialize the shader program: ${gl.getProgramInfoLog(shaderProgram)}`);
        return null;
    }

    return shaderProgram;
}

function hueToRgb(p, q, t) {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1/6) return p + (q - p) * 6 * t;
    if (t < 1/2) return q;
    if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
    return p;
}

function hslToRgb(h, s, l) {
    let r, g, b;

    if (s === 0) {
        r = g = b = l; // achromatic
    } else {
        const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
        const p = 2 * l - q;
        r = hueToRgb(p, q, h + 1/3);
        g = hueToRgb(p, q, h);
        b = hueToRgb(p, q, h - 1/3);
    }

    return [r, g, b];
}

const text_width = 700;
const text_height = 300;

function getTextImage(text) {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    if (!ctx) return null;

    canvas.width = text_width;
    canvas.height = text_height;

    ctx.translate(canvas.width / 2, canvas.height / 2);

    ctx.fillStyle = '#ffffff';
    ctx.strokeStyle = 'white';
    ctx.font = 'bold 192px sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText(text, 0, 96);

    return ctx.getImageData(0, 0, canvas.width, canvas.height);
}

(function() {
    const canvas = document.getElementById('render');
    if (!canvas) {
        alert('Нет канваса, ватахелл');
        return;
    }
    const gl = canvas.getContext('webgl');
    if (!gl) {
        alert('Нету WebGL 1, ты старый!');
        return;
    }

    const shader_pos_attrib = 0;
    const shader_uv_attrib = 1;

    const locations = [
        { location: shader_pos_attrib, name: 'posAttr' },
        { location: shader_uv_attrib, name: 'uvAttr' },
    ];

    const shaderBack = initShader(gl, vertexSourceBack, fragmentSourceBack, locations);
    if (!shaderBack) {
        console.error('Shader compilation failed! See console for details');
        return;
    }

    const shaderFront = initShader(gl, vertexSourceFront, fragmentSourceFront, locations);

    if (!shaderFront) {
        console.error('Shader compilation failed! See console for details');
        return;
    }

    const shaderText = initShader(gl, vertexSourceText, fragmentSourceText, locations);

    if (!shaderText) {
        console.error('Shader compilation failed! See console for details');
        return;
    }

    const canvas_texture = getTextImage('Абоба');
    if (!canvas_texture) {
        console.error('Canvas texture load failed!');
        return;
    }

    const text_texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, text_texture);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, canvas_texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    // Create back framebuffer
    const back_buffer = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, back_buffer);

    let back_buffer_texture = make_back_buffer_texture();
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, back_buffer_texture, 0);

    if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) != gl.FRAMEBUFFER_COMPLETE) {
        console.error('Framebuffer is not ready!');
        return;
    }

    gl.bindFramebuffer(gl.FRAMEBUFFER, null);

    const text_size = { x: text_width / (canvas.width * 2), y: text_height / (canvas.height * 2)};

    const text_buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, text_buffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
        -1.0 * text_size.x,  1.0 * text_size.y, 0.0, 1.0,
        -1.0 * text_size.x, -1.0 * text_size.y, 0.0, 0.0,
         1.0 * text_size.x,  1.0 * text_size.y, 1.0, 1.0,
         1.0 * text_size.x, -1.0 * text_size.y, 1.0, 0.0,
    ]), gl.STATIC_DRAW);

    // Make some buffers
    const quad_buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, quad_buffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
        -1.0,  1.0, 0.0, 1.0,
        -1.0, -1.0, 0.0, 0.0,
         1.0,  1.0, 1.0, 1.0,
         1.0, -1.0, 1.0, 0.0,
    ]), gl.STATIC_DRAW);

    gl.enableVertexAttribArray(shader_pos_attrib);
    gl.enableVertexAttribArray(shader_uv_attrib);

    // Shader preparations
    gl.useProgram(shaderBack);

    const shader_time_uniform = gl.getUniformLocation(shaderBack, 'time');
    const shader_bg_color_uniform = gl.getUniformLocation(shaderBack, 'bg_col');
    const shader_fg_color_uniform = gl.getUniformLocation(shaderBack, 'fg_col');
    const shader_aspect_uniform = gl.getUniformLocation(shaderBack, 'aspect');

    gl.uniform3f(shader_bg_color_uniform, ...hslToRgb(Math.random(), Math.random(), 0.1));
    gl.uniform3f(shader_fg_color_uniform, ...hslToRgb(Math.random(), Math.random(), 0.7));
    gl.uniform1f(shader_aspect_uniform, canvas.width / canvas.height);

    gl.useProgram(shaderFront);

    const shader_back_buffer_uniform = gl.getUniformLocation(shaderFront, 'back_buffer');
    const shader_front_time_uniform = gl.getUniformLocation(shaderFront, 'time');

    gl.useProgram(shaderText);

    const shader_text_uniform = gl.getUniformLocation(shaderText, 'text');
    
    function assign_attributes() {
        // Assign buffer to pos attribute
        gl.vertexAttribPointer(
            shader_pos_attrib, // Set for pos attribute
            2, // vec2
            gl.FLOAT, // vec2 uses floats
            false, // Don't normalize values
            16, // stride - how many bytes to get from one set of values to the next including the values itself
            0, // offset in bytes, or pointer if gl.ARRAY_BUFFER is not bound
        );

        gl.vertexAttribPointer(
            shader_uv_attrib, // Set for uv attribute
            2, // vec2
            gl.FLOAT, // vec2 uses floats
            false, // Don't normalize values
            16, // stride - how many bytes to get from one set of values to the next including the values itself
            8, // offset, or pointer if gl.ARRAY_BUFFER is not bound
        );
    }

    function make_back_buffer_texture() {
        const texture = gl.createTexture();
        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, texture);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, canvas.width, canvas.height, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

        return texture;
    }

    function resize() {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;

        gl.useProgram(shaderBack);
        gl.viewport(0, 0, canvas.width, canvas.height);
        if (shader_aspect_uniform) {
            gl.uniform1f(shader_aspect_uniform, canvas.width / canvas.height);
        }

        gl.bindFramebuffer(gl.FRAMEBUFFER, back_buffer);

        gl.deleteTexture(back_buffer_texture);
        back_buffer_texture = make_back_buffer_texture();
        gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, back_buffer_texture, 0);

        gl.bindFramebuffer(gl.FRAMEBUFFER, null);

        const text_size = { x: text_width / canvas.width, y: text_height / canvas.height};

        gl.bindBuffer(gl.ARRAY_BUFFER, text_buffer);
        gl.bufferSubData(gl.ARRAY_BUFFER, 0, new Float32Array([
            -1.0 * text_size.x,  1.0 * text_size.y, 0.0, 1.0,
            -1.0 * text_size.x, -1.0 * text_size.y, 0.0, 0.0,
             1.0 * text_size.x,  1.0 * text_size.y, 1.0, 1.0,
             1.0 * text_size.x, -1.0 * text_size.y, 1.0, 0.0,
        ]));
    }

    function draw(time) {
        // First pass
        gl.bindFramebuffer(gl.FRAMEBUFFER, back_buffer);

        gl.clearColor(0.6, 0.7, 0.8, 1.0);
        gl.clear(gl.COLOR_BUFFER_BIT);

        gl.useProgram(shaderBack);

        gl.uniform1f(shader_time_uniform, time / 1000);

        gl.bindBuffer(gl.ARRAY_BUFFER, quad_buffer);
        assign_attributes();
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

        gl.useProgram(shaderText);

        gl.blendFunc(gl.ONE_MINUS_DST_COLOR, gl.ONE_MINUS_SRC_ALPHA);
        gl.enable(gl.BLEND);

        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, text_texture);
        gl.uniform1i(shader_text_uniform, 0);

        gl.bindBuffer(gl.ARRAY_BUFFER, text_buffer);
        assign_attributes();
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

        gl.disable(gl.BLEND);

        // Second pass
        gl.bindFramebuffer(gl.FRAMEBUFFER, null);

        gl.useProgram(shaderFront);

        gl.clearColor(0.6, 0.7, 0.8, 1.0);
        gl.clear(gl.COLOR_BUFFER_BIT);

        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, back_buffer_texture);

        gl.uniform1i(shader_back_buffer_uniform, 0);
        gl.uniform1f(shader_front_time_uniform, time / 1000);

        gl.bindBuffer(gl.ARRAY_BUFFER, quad_buffer);
        assign_attributes();
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

        requestAnimationFrame(draw);
    }

    window.addEventListener('resize', resize);
    resize();
    draw(0);
})();
