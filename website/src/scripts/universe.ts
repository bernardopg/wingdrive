// "One library" scene: a glowing WingDrive core with drives and clouds in orbit,
// joined by beams that carry data pulses. Drag to spin; it settles back slowly.
import {
	AdditiveBlending,
	BufferGeometry,
	CatmullRomCurve3,
	Color,
	DoubleSide,
	Group,
	IcosahedronGeometry,
	Line,
	LineBasicMaterial,
	Mesh,
	MeshBasicMaterial,
	MeshPhysicalMaterial,
	MeshStandardMaterial,
	Object3D,
	PerspectiveCamera,
	PMREMGenerator,
	RingGeometry,
	Scene,
	ShaderMaterial,
	SphereGeometry,
	Vector3,
	WebGLRenderer,
	BoxGeometry,
	CylinderGeometry,
	EdgesGeometry,
	LineSegments,
} from "three";
import { RoundedBoxGeometry } from "three/addons/geometries/RoundedBoxGeometry.js";
import { RoomEnvironment } from "three/addons/environments/RoomEnvironment.js";

type Labelled = { obj: Object3D; el: HTMLElement };

export function mountUniverse(root: HTMLElement) {
	const canvas = root.querySelector("canvas")!;
	const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
	let renderer: WebGLRenderer;
	try {
		renderer = new WebGLRenderer({ canvas, alpha: true, antialias: true });
	} catch {
		root.classList.add("no-webgl");
		return;
	}
	renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
	const scene = new Scene();
	const pmrem = new PMREMGenerator(renderer);
	scene.environment = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
	scene.environmentIntensity = 0.55;
	const camera = new PerspectiveCamera(38, 1, 0.1, 100);
	camera.position.set(0, 2.4, 12);
	camera.lookAt(0, 0, 0);

	const world = new Group();
	scene.add(world);

	// Core: iridescent glass sphere around an emissive heart.
	const core = new Group();
	const heart = new Mesh(new IcosahedronGeometry(0.62, 4), new MeshBasicMaterial({ color: new Color("#a78bfa") }));
	const shell = new Mesh(
		new SphereGeometry(1.05, 64, 64),
		new MeshPhysicalMaterial({
			color: new Color("#8b5cf6"),
			roughness: 0.05,
			metalness: 0,
			transmission: 1,
			thickness: 0.8,
			ior: 1.35,
			iridescence: 1,
			iridescenceIOR: 1.6,
			transparent: true,
			opacity: 0.9,
		}),
	);
	const halo = new Mesh(
		new SphereGeometry(1.9, 48, 48),
		new ShaderMaterial({
			transparent: true,
			depthWrite: false,
			blending: AdditiveBlending,
			side: DoubleSide,
			uniforms: { c: { value: new Color("#7c3aed") } },
			vertexShader: `varying vec3 vN; varying vec3 vV; void main(){ vec4 mv = modelViewMatrix*vec4(position,1.); vN = normalize(normalMatrix*normal); vV = normalize(-mv.xyz); gl_Position = projectionMatrix*mv; }`,
			fragmentShader: `uniform vec3 c; varying vec3 vN; varying vec3 vV; void main(){ float f = pow(1.0-abs(dot(vN,vV)), 2.5); gl_FragColor = vec4(c*1.6, f*0.75); }`,
		}),
	);
	const wire = new LineSegments(
		new EdgesGeometry(new IcosahedronGeometry(1.32, 1)),
		new LineBasicMaterial({ color: "#c4b5fd", transparent: true, opacity: 0.22 }),
	);
	core.add(heart, shell, halo, wire);
	world.add(core);

	// Orbit rings.
	for (const [r, tilt] of [
		[2.9, 0.2],
		[4.1, -0.12],
	] as const) {
		const ring = new Mesh(
			new RingGeometry(r - 0.006, r + 0.006, 160),
			new MeshBasicMaterial({ color: "#a78bfa", transparent: true, opacity: 0.22, side: DoubleSide }),
		);
		ring.rotation.x = Math.PI / 2 + tilt;
		world.add(ring);
	}

	const shellMat = (hex: string) =>
		new MeshStandardMaterial({ color: new Color(hex), metalness: 0.85, roughness: 0.22 });
	const glowMat = new MeshBasicMaterial({ color: new Color("#c4b5fd") });
	const edgeMat = new LineBasicMaterial({ color: "#a78bfa", transparent: true, opacity: 0.55 });

	const withEdges = (m: Mesh) => {
		m.add(new LineSegments(new EdgesGeometry(m.geometry, 30), edgeMat));
		return m;
	};

	const ssd = new Group();
	ssd.add(withEdges(new Mesh(new RoundedBoxGeometry(1.25, 0.12, 0.8, 4, 0.05), shellMat("#2a1b55"))));
	const ssdLed = new Mesh(new BoxGeometry(0.5, 0.02, 0.05), glowMat);
	ssdLed.position.set(0, 0.07, -0.2);
	ssd.add(ssdLed);

	const hdd = new Group();
	hdd.add(withEdges(new Mesh(new RoundedBoxGeometry(0.95, 0.32, 1.3, 5, 0.12), shellMat("#1d1440"))));
	const hddLed = new Mesh(new BoxGeometry(0.06, 0.06, 0.02), glowMat);
	hddLed.position.set(0.3, 0, 0.66);
	hdd.add(hddLed);

	const usb = new Group();
	usb.add(withEdges(new Mesh(new RoundedBoxGeometry(0.34, 0.14, 0.9, 4, 0.06), shellMat("#3b2780"))));
	const plug = new Mesh(new BoxGeometry(0.24, 0.09, 0.3), new MeshStandardMaterial({ color: "#d8d4ee", metalness: 1, roughness: 0.25 }));
	plug.position.z = 0.58;
	usb.add(plug);

	const nas = new Group();
	nas.add(withEdges(new Mesh(new RoundedBoxGeometry(0.9, 0.85, 0.9, 5, 0.1), shellMat("#1a1238"))));
	for (let i = 0; i < 3; i++) {
		const bay = new Mesh(new BoxGeometry(0.16, 0.6, 0.02), new MeshBasicMaterial({ color: i === 1 ? "#a78bfa" : "#5b3fd1" }));
		bay.position.set(-0.24 + i * 0.24, 0, 0.46);
		nas.add(bay);
	}

	const cloud = (hex: string) => {
		const g = new Group();
		const mat = new MeshPhysicalMaterial({
			color: new Color(hex),
			roughness: 0.35,
			transmission: 0.6,
			thickness: 0.5,
			transparent: true,
			opacity: 0.95,
			emissive: new Color(hex),
			emissiveIntensity: 0.25,
		});
		for (const [x, y, s] of [
			[-0.35, 0, 0.32],
			[0, 0.12, 0.42],
			[0.36, 0, 0.3],
			[0.1, -0.08, 0.3],
		] as const) {
			const b = new Mesh(new SphereGeometry(s, 32, 32), mat);
			b.position.set(x, y, 0);
			g.add(b);
		}
		return g;
	};

	type Orbiter = { obj: Group; r: number; speed: number; phase: number; y: number; tilt: number; spin: number };
	const orbiters: Orbiter[] = [
		{ obj: ssd, r: 2.9, speed: 0.16, phase: 0.2, y: 0.2, tilt: 0.2, spin: 0.4 },
		{ obj: hdd, r: 2.9, speed: 0.16, phase: 2.3, y: 0, tilt: 0.2, spin: -0.3 },
		{ obj: usb, r: 2.9, speed: 0.16, phase: 4.3, y: -0.1, tilt: 0.2, spin: 0.6 },
		{ obj: nas, r: 4.1, speed: -0.1, phase: 1.0, y: 0.3, tilt: -0.12, spin: 0.2 },
		{ obj: cloud("#8b5cf6"), r: 4.1, speed: -0.1, phase: 3.1, y: 0.7, tilt: -0.12, spin: 0 },
		{ obj: cloud("#6366f1"), r: 4.1, speed: -0.1, phase: 5.0, y: 0.5, tilt: -0.12, spin: 0 },
	];
	for (const o of orbiters) {
		o.obj.scale.setScalar(1.45);
		world.add(o.obj);
	}

	// Beams with travelling pulses from each orbiter to the core.
	const beamMat = new LineBasicMaterial({ color: "#a78bfa", transparent: true, opacity: 0.28, blending: AdditiveBlending });
	const pulseGeo = new SphereGeometry(0.05, 12, 12);
	const pulseMat = new MeshBasicMaterial({ color: "#e9d5ff", blending: AdditiveBlending, transparent: true });
	const beams = orbiters.map((o) => {
		const line = new Line(new BufferGeometry(), beamMat);
		world.add(line);
		const pulses = [0, 0.33, 0.66].map((off) => {
			const p = new Mesh(pulseGeo, pulseMat);
			world.add(p);
			return { p, off };
		});
		return { o, line, pulses, curve: new CatmullRomCurve3([new Vector3(), new Vector3(), new Vector3()]) };
	});

	const labels: Labelled[] = [];
	root.querySelectorAll<HTMLElement>("[data-label]").forEach((el) => {
		const idx = Number(el.dataset.label);
		const o = orbiters[idx];
		if (o) labels.push({ obj: o.obj, el });
	});

	// Drag to rotate with inertia.
	let dragging = false;
	let lastX = 0;
	let lastY = 0;
	let velY = 0;
	let rotX = 0.12;
	let userRot = 0;
	canvas.addEventListener("pointerdown", (e) => {
		dragging = true;
		lastX = e.clientX;
		lastY = e.clientY;
		canvas.setPointerCapture(e.pointerId);
		root.classList.add("grabbing");
	});
	canvas.addEventListener("pointermove", (e) => {
		if (!dragging) return;
		const dx = e.clientX - lastX;
		const dy = e.clientY - lastY;
		lastX = e.clientX;
		lastY = e.clientY;
		velY = dx * 0.005;
		userRot += velY;
		rotX = Math.max(-0.3, Math.min(0.6, rotX + dy * 0.003));
	});
	const end = () => {
		dragging = false;
		root.classList.remove("grabbing");
	};
	canvas.addEventListener("pointerup", end);
	canvas.addEventListener("pointercancel", end);

	const resize = () => {
		const w = canvas.clientWidth;
		const h = canvas.clientHeight;
		renderer.setSize(w, h, false);
		camera.aspect = w / h;
		// Fit the outer orbit (radius 4.1 plus object size) inside the frame on both axes.
		const R = 4.1 + 1.3;
		const vFov = (camera.fov * Math.PI) / 180;
		const hFov = 2 * Math.atan(Math.tan(vFov / 2) * camera.aspect);
		const dist = Math.max(R / Math.tan(vFov / 2) * 0.62, R / Math.tan(hFov / 2)) * 1.08;
		const lift = canvas.clientWidth < 640 ? 0.5 : 0.38;
		camera.position.set(0, dist * lift, dist * (1 - lift * 0.15));
		camera.lookAt(0, 0, 0);
		camera.updateProjectionMatrix();
	};
	resize();
	new ResizeObserver(resize).observe(canvas);

	let visible = false;
	new IntersectionObserver(([e]) => (visible = e.isIntersecting), { rootMargin: "100px" }).observe(canvas);

	const tmp = new Vector3();
	const t0 = performance.now();
	const tick = () => {
		requestAnimationFrame(tick);
		if (!visible) return;
		const time = reduce ? 4 : (performance.now() - t0) / 1000;
		if (!dragging) {
			velY *= 0.94;
			userRot += velY;
		}
		world.rotation.y = userRot + (reduce ? 0 : time * 0.05);
		world.rotation.x = rotX;
		heart.scale.setScalar(1 + Math.sin(time * 2.2) * 0.05);
		wire.rotation.y = time * 0.25;
		wire.rotation.x = time * 0.15;

		for (const o of orbiters) {
			const a = o.phase + time * o.speed;
			o.obj.position.set(Math.cos(a) * o.r, o.y + Math.sin(a) * o.r * Math.sin(o.tilt) + Math.sin(time + o.phase) * 0.12, Math.sin(a) * o.r);
			o.obj.rotation.y = time * o.spin + o.phase;
			o.obj.rotation.x = Math.sin(time * 0.6 + o.phase) * 0.25;
		}
		for (const b of beams) {
			const end = b.o.obj.position;
			const mid = end.clone().multiplyScalar(0.5);
			mid.y += 0.9;
			b.curve.points[0].set(0, 0, 0);
			b.curve.points[1].copy(mid);
			b.curve.points[2].copy(end);
			b.line.geometry.setFromPoints(b.curve.getPoints(32));
			for (const { p, off } of b.pulses) {
				const u = (time * 0.35 + off) % 1;
				b.curve.getPoint(1 - u, p.position);
				p.scale.setScalar(0.6 + Math.sin(u * Math.PI) * 0.8);
			}
		}

		renderer.render(scene, camera);

		const w = canvas.clientWidth;
		const h = canvas.clientHeight;
		for (const l of labels) {
			l.obj.getWorldPosition(tmp);
			tmp.y += 0.75;
			tmp.project(camera);
			const lw = l.el.offsetWidth / 2 + 4;
			const x = Math.min(w - lw, Math.max(lw, (tmp.x * 0.5 + 0.5) * w));
			const y = Math.max(l.el.offsetHeight + 4, (-tmp.y * 0.5 + 0.5) * h);
			l.el.style.transform = `translate(${x}px, ${y}px) translate(-50%, -100%)`;
			l.el.style.opacity = tmp.z < 1 ? "1" : "0";
		}
	};
	tick();
}
