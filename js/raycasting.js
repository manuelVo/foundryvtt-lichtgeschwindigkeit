import init, * as Lichtgeschwindigkeit from "../wasm/lichtgeschwindigkeit.js";

init().then(() => {
	SightLayer.computeSight = wasmComputePolygon;
	WallsLayer.prototype.computePolygon = wasmComputePolygon;
	Hooks.on("canvasInit", Lichtgeschwindigkeit.wipeCache);
	Hooks.on("createWall", Lichtgeschwindigkeit.wipeCache);
	Hooks.on("updateWall", Lichtgeschwindigkeit.wipeCache);
	Hooks.on("deleteWall", Lichtgeschwindigkeit.wipeCache);
	window.lichtgeschwindigkeit = {
		build_scene,
		generate_test,
	}
});

function wasmComputePolygon(origin, radius, { type = "sight", angle = 360, density = 6, rotation = 0, unrestricted = false } = {}) {
	let debugEnabled = CONFIG.debug.sightRays;
	// The maximum ray distance needs to reach all areas of the canvas
	let d = canvas.dimensions;
	const dx = Math.max(origin.x, d.width - origin.x);
	const dy = Math.max(origin.y, d.height - origin.y);
	const distance = Math.max(radius, Math.hypot(dx, dy));

	let internals = null;
	if (debugEnabled)
		internals = {};

	let walls;
	if (unrestricted) {
		walls = [];
	}
	else {
		walls = canvas.walls.placeables;
	}

	function logParams(force, error_fn) {
		rustifyParams(walls, type, origin, radius, distance, density, angle, rotation, force, error_fn);
	}

	if (debugEnabled)
		logParams();

	let sight;
	try {
		sight = Lichtgeschwindigkeit.computePolygon(walls, type, origin, radius, distance, density, angle, rotation, internals);
	}
	catch (e) {
		console.error(e);
		console.error("Data to reproduce the error (please always include this in bug reports!):");
		logParams(true, console.error);
		throw e;
	}

	// Lichtgeschwindigkeit improves the speed of PIXI.Polygon.contains.
	// Those improvements outperform the improvements done by SourcePolygon.
	// As a result we don't construct SourcePolygon here.
	const los = new PIXI.Polygon(...sight.los);
	const fov = new PIXI.Polygon(...sight.fov);

	if (debugEnabled) {
		_visualizeSight(internals.endpoints, origin, radius, distance, los, fov, sight.los, true);
	}

	return { rays: null, los, fov };
}

function rustifyParams(walls, type, origin, radius, distance, density, angle, rotation, force = false, error_fn = console.warn) {
	/*if (!force) {
		if (canvas.tokens.controlled.length === 0)
			return;
		if (Math.abs(origin.x - canvas.tokens.controlled[0].data.x) > 50 || Math.abs(origin.y - canvas.tokens.controlled[0].data.y) > 50)
			return;
	}*/
	error_fn(Lichtgeschwindigkeit.serializeData(walls, type, origin, radius, distance, density, angle, rotation));
}

function _visualizeSight(endpoints, origin, radius, distance, los, fov, tangentPoints, clear = true) {
	/*if (canvas.tokens.controlled.length === 0)
		return;
	if (Math.abs(origin.x - canvas.tokens.controlled[0].data.x) > 50 || Math.abs(origin.y - canvas.tokens.controlled[0].data.y) > 50)
		return;*/
	const debug = canvas.controls.debug;
	if (!debug)
		return;
	if (clear)
		debug.clear();

	// Relevant polygons
	debug.lineStyle(0).beginFill(0x66FFFF, 0.2).drawShape(los);
	debug.beginFill(0xFF66FF, 0.2).drawShape(fov).endFill();

	// Tested endpoints
	if (!endpoints)
		endpoints = [];

	for (const endpoint of endpoints) {
		const color = endpoint.isIntersection ? 0xFF0000 : 0x00FFFF;
		debug.lineStyle(0).beginFill(color, 1.0).drawCircle(endpoint.x, endpoint.y, 9).endFill();
	}

	for (const point of tangentPoints) {
		debug.lineStyle(2, 0xDDFF00).drawCircle(point.x, point.y, 5);
	}

	// Walls
	for (const wall of canvas.walls.placeables) {
		debug.lineStyle(3, 0x000000).moveTo(wall.data.c[0], wall.data.c[1]).lineTo(wall.data.c[2], wall.data.c[3]);
	}

	// Sight range
	debug.lineStyle(1, 0xFF0000).drawCircle(origin.x, origin.y, radius);

	// Cast rays
	for (const endpoint of endpoints) {
		debug.lineStyle(1, 0x00FF00).moveTo(origin.x, origin.y).lineTo(origin.x - Math.cos(endpoint.angle) * distance, origin.y - Math.sin(endpoint.angle) * distance);
	}
}

function build_scene() {
	new Dialog({
		content: "<textarea id='lichtgeschwindigkeit-debug-input'></textarea>",
		buttons: {
			ok: {
				icon: '<i class="fas fa-check"></i>',
				callback: html => {
					let data = document.getElementById("lichtgeschwindigkeit-debug-input").value;
					data = Lichtgeschwindigkeit.deserializeData(data);
					import("./scene_builder.js").then((module) => module.build_scene(data));
				}
			}
		}
	}).render(true);
}

function generate_test() {
	new Dialog({
		content: "<textarea id='lichtgeschwindigkeit-debug-input'></textarea>",
		buttons: {
			ok: {
				icon: '<i class="fas fa-check"></i>',
				callback: html => {
					let data = document.getElementById("lichtgeschwindigkeit-debug-input").value;
					console.warn(Lichtgeschwindigkeit.generateTest(data));
				}
			}
		}
	}).render(true);
}
