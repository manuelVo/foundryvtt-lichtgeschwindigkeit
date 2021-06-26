let originalContains = undefined;

Hooks.on("init", () => {
	originalContains = PIXI.Polygon.prototype.contains;
	PIXI.Polygon.prototype.contains = contains;
});

function contains(x, y) {
	if (this.top === undefined) {
		// Construct the bounding box of the polygon
		const length = this.points.length / 2;
		this.left = Infinity;
		this.right = -Infinity;
		this.top = Infinity;
		this.bottom = -Infinity;
		for (let i = 0;i < length;i++) {
			const px = this.points[i * 2];
			const py  = this.points[i * 2 + 1];
			this.left = Math.min(px, this.left);
			this.right = Math.max(px, this.right);
			this.top = Math.min(py, this.top);
			this.bottom = Math.max(py, this.bottom);
		}
	}

	// Check if the polygon is within the bounding box
	if (x < this.left || x > this.right || y < this.top || y > this.bottom) {
		return false;
	}

	return originalContains.call(this, x, y);
}
