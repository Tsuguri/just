class nojs {
    constructor() {
        this.changed = false;
        this.last = Time.elapsed;
        this.last_dest = 1.0;
        this.mesh = null;
    }

    update() {
        let posit = this.go.position;
        console.log(posit.x, " ", posit.y, " ", posit.z);


        let pos = new Math.Vector(0.1, 0.2, 0.3);
        pos.x = 12.0;
        console.log("test: ", pos.x, " ", pos.y, " ", pos.z);
        if (!this.mesh) {
            this.mesh = Resources.getMesh("monkey");
        }
        if (this.changed === false && Input.isKeyboardKeyPressed("A")) {
            this.go.name = "heh";
            this.changed = true;
        }
        if (Time.elapsed > this.last + 1.0) {
            this.last = Time.elapsed;
            let n = World.createGameObject();
            const pos = new Math.Vector(this.last, Math.Sin(this.last), 0.0);
            n.position = pos;
            n.name = "heh2";
            n.setRenderable(this.mesh);
            n.setScript("test2");
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
