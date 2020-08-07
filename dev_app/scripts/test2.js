class nojs {
    constructor() {
        this.changed = false;
        this.last = Time.elapsed;
        this.last_dest = 1.0;
        this.mesh = null;
        this.some = new Math2.Vector3(0.0, 0.0, 0.0);
        this.someLol = new Math2.Vector2(0.0, 1.0);
    }

    dotto() {
        console.log("success");
    }

    update() {
        let posit = this.go.position;
        //console.log(posit.x, " ", posit.y, " ", posit.z);
        this.some.x = Math2.Sin(Time.elapsed);
        World.setCameraPosition(this.some);


        let pos = new Math2.Vector3(0.1, 0.2, 0.3);
        pos.x = 12.0;
        //console.log("test: ", pos.x, " ", pos.y, " ", pos.z)
        let pos2 = this.go.globalPosition;
        //console.log("test: ", pos2.x, " ", pos2.y, " ", pos2.z)

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
            const pos = new Math2.Vector3(this.last, Math2.Sin(this.last), 5.0);
            n.position = pos;
            n.name = "heh2";
            n.setRenderable(this.mesh);
            n.setScript("test_script");


        }
        if (Time.elapsed > this.last_dest + 2.0) {
            this.last_dest = Time.elapsed;
            //console.log("destroying");
            let n = World.findByName("heh");
            if (n.length > 0) {
                n[0].destroy();
            }
        }
    }
}
