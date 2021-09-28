import init, * as Lichtgeschwindigkeit from "../wasm/lichtgeschwindigkeit.js";

let wallHeightEnabled;

init().then(() => {
	SightLayer.computeSight = wasmComputePolygon;
	WallsLayer.prototype.computePolygon = wasmComputePolygon;
	Hooks.on("canvasInit", wipeCache);
	Hooks.on("canvasReady", wipeCache);
	Hooks.on("createWall", wipeCache);
	Hooks.on("updateWall", wipeCache);
	Hooks.on("deleteWall", wipeCache);
	Hooks.on("createTile", wipeCache);
	Hooks.on("updateTile", wipeCache);
	Hooks.on("deleteTile", wipeCache);
	hookUpdateOcclusion();
	window.lichtgeschwindigkeit = {
		build_scene,
		generate_test,
	}
});

Hooks.once("init", () => {
	// This can affect the outcome of vision calculations, so we wipe the cache just to be sure
	wallHeightEnabled = game.modules.get("wall-height")?.active;
	wipeCache();
});

let cache = undefined;
let emptyCache = undefined;

function wipeCache() {
	if (cache)
		Lichtgeschwindigkeit.wipeCache(cache);
	cache = undefined;
}

function hookUpdateOcclusion() {
	let original = Tile.prototype.updateOcclusion;
	Tile.prototype.updateOcclusion = function(tokens) {
		const oldOcclusion = this.occluded;
		original.call(this, tokens);
		if (cache && oldOcclusion != this.occluded && this.data.occlusion.mode === CONST.TILE_OCCLUSION_MODES.ROOF) {
			Lichtgeschwindigkeit.updateOcclusion(cache, this.id, this.occluded);
		}
	}
}

function wasmComputePolygon(origin, radius, { type = "sight", angle = 360, density = 6, rotation = 0, unrestricted = false } = {}) {
	// TODO This hotfix may no longer be necessary in foundry 9
	if (type === "sight")
		radius = Math.max(radius, canvas.dimensions.size >> 1); // canvas.dimensions.size >> 1 is a fast method of calculating canvas.dimensions.size / 2

	let debugEnabled = CONFIG.debug.sightRays;
	// The maximum ray distance needs to reach all areas of the canvas
	let d = canvas.dimensions;
	const dx = Math.max(origin.x, d.width - origin.x);
	const dy = Math.max(origin.y, d.height - origin.y);
	const distance = Math.max(radius, Math.hypot(dx, dy));
	const height = game.currentTokenElevation ?? 0;

	let internals = null;
	if (debugEnabled)
		internals = {};

	let cacheRef;
	if (unrestricted) {
		if (!emptyCache)
			emptyCache = Lichtgeschwindigkeit.buildCache([], wallHeightEnabled);
		cacheRef = emptyCache;
	}
	else {
		if (!cache)
			cache = Lichtgeschwindigkeit.buildCache(canvas.walls.placeables, wallHeightEnabled);
		cacheRef = cache
	}

	function logParams(force, error_fn) {
		rustifyParams(cacheRef, origin, height, radius, distance, density, angle, rotation, type, force, error_fn);
	}

	if (debugEnabled)
		logParams();

	let sight;
	try {
		sight = Lichtgeschwindigkeit.computePolygon(cacheRef, origin, height, radius, distance, density, angle, rotation, type, internals);
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

function rustifyParams(cache, origin, height, radius, distance, density, angle, rotation, type, force = false, error_fn = console.warn) {
	/*if (!force) {
		if (canvas.tokens.controlled.length === 0)
			return;
		if (Math.abs(origin.x - canvas.tokens.controlled[0].data.x) > 50 || Math.abs(origin.y - canvas.tokens.controlled[0].data.y) > 50)
			return;
	}*/
	error_fn(Lichtgeschwindigkeit.serializeData(cache, origin, height, radius, distance, density, angle, rotation, type));
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
