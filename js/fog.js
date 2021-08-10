Hooks.on("init", () => {
	const [ major, minor, bugfix ] = game.data.version.split(".");
	if (major == 0 && minor == 7)
		return;
	if (major == 0 && minor == 8 && bugfix < 7) {
		return;
	}
	SightLayer.prototype.commitFog = commitFog;
});

let recycledRenderTexture = undefined;

function getRecycledRenderTexture(d) {
	if (recycledRenderTexture) {
		if (recycledRenderTexture.baseTexture && recycledRenderTexture.width === d.width && recycledRenderTexture.height === d.height && recycledRenderTexture.resolution === d.resolution) {
			const tex = recycledRenderTexture;
			recycledRenderTexture = undefined;
			return tex;
		}
		recycledRenderTexture.destroy(true);
	}
	const tex = PIXI.RenderTexture.create(d);
	return tex;
}

function recycleRenderTexture(tex) {
	if (!(tex instanceof PIXI.RenderTexture)) {
		tex.destroy(true);
		return;
	}
	if (recycledRenderTexture) {
		recycledRenderTexture.destroy(true);
	}
	recycledRenderTexture = tex;
}

function commitFog() {
	if (CONFIG.debug.fog) console.debug("SightLayer | Committing fog exploration to render texture.");
	this._fogUpdates = 0;

	// Protect against an invalid render texture
	if (!this.saved.texture.valid) {
		this.saved.texture = PIXI.Texture.EMPTY;
	}

	// Create a staging texture and render the entire fog container to it
	const d = canvas.dimensions;
	const tex = getRecycledRenderTexture(this._fogResolution);
	const transform = new PIXI.Matrix(1, 0, 0, 1, -d.paddingX, -d.paddingY);

	// Render the texture (temporarily disabling the masking rectangle)
	canvas.app.renderer.render(this.revealed, tex, undefined, transform);

	// Swap the staging texture to the rendered Sprite
	recycleRenderTexture(this.saved.texture);
	this.saved.texture = tex;
	this.pending.removeChildren().forEach(c => this._recycleVisionContainer(c));

	// Record that fog was updated and schedule a save
	this._fogUpdated = true;
	this.debounceSaveFog();
}
