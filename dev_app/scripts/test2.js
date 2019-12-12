class nojs {
    constructor() {
        this.changed = false;
        this.last = 0.0;
        this.last_dest = 1.0;
        this.mesh = null;
    }

    update() {
        if (!this.mesh) {
            this.mesh = Resources.getMesh("monkey");
        }
        if (this.changed === false && Input.isKeyboardKeyPressed("A")) {
            this.go.name = "heh";
            this.changed = true;
        }
        if (Time.elapsed > this.last + 2.0) {
            this.last = Time.elapsed;
            let n = World.createGameObject();
            const pos = new Math.Vector(0.0, this.last, 0.0);
            n.parent = this.go;
            n.position = pos;
            n.name = "heh2";
            n.setRenderable(this.mesh);
        }
        if (Time.elapsed > this.last_dest + 2.0) {
            this.last_dest = Time.elapsed;
            console.log("destroying");
            let n = World.findByName("heh");
            if (n.length > 0) {
                n[0].destroy();
            }
        }
    }
}
