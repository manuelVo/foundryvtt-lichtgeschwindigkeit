export async function build_scene(data) {
	const points = data.walls.map(wall => [wall.p1, wall.p2]).flat();
	let width = points.map(p => p.x).reduce((max, current) => max > current ? max : current, data.origin.x);
	let height = points.map(p => p.y).reduce((max, current) => max > current ? max : current, data.origin.y);
	const now = new Date();
	const name = `${now.getFullYear()}-${now.getMonth()}-${now.getDay()} ${now.getHours()}:${now.getMinutes()}:${now.getSeconds()}`;
	const folder = await getOrCreateFolder();
	const sceneData = [{
		name: name,
		active: false,
		navigation: true,
		width: width,
		height: height,
		globalLight: false,
		grid: 100,
		gridDistance: 100,
		gridType: CONST.GRID_TYPES.GRIDLESS,
		initial: { x: data.origin.x, y: data.origin.y, scale: 0.5 },
		tokenVision: true,
		folder: folder.id,
	}];
	const wallData = data.walls.map(wall => {
		return {
			c: [wall.p1.x, wall.p1.y, wall.p2.x, wall.p2.y],
			door: wall.door,
			ds: wall.ds,
			sense: wall.sense,
			dir: wall.dir,
		};
	});
	const tokenData = [{
		actorId: "",
		name: "Mr. Bug",
		actorLink: false,
		brightSight: data.radius - 50,
		brightLight: 0,
		dimSight: 0,
		dimLight: 0,
		height: 1,
		width: 1,
		scale: 1,
		hidden: false,
		sightAngle: data.angle,
		rotation: data.rotation,
		x: data.origin.x - 50,
		y: data.origin.y - 50,
	}];
	let scene = await Scene.create(sceneData, { renderSheet: false });
	console.warn(scene);
	if (["0.7.9", "0.7.10"].includes(game.data.version)) {
		await scene.createEmbeddedEntity("Wall", wallData);
		await scene.createEmbeddedEntity("Token", tokenData);
	}
	else {
		scene = scene[0];
		await scene.createEmbeddedDocuments("Wall", wallData);
		await scene.createEmbeddedDocuments("Token", tokenData);
	}
	await scene.activate();

}

function getOrCreateFolder() {
	const folderName = "Lichtgeschwindigkeit Gen.";
	let folder = Array.from(game.folders.values()).find(folder => folder.data.type === "Scene" && folder.data.name === folderName);
	if (folder)
		return folder;
	const folderData = [{
		name: folderName,
		color: "#081d72",
		parent: null,
		sort: null,
		sorting: "a",
		type: "Scene",
	}];
	return Folder.create(folderData);
}
