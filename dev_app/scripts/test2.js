test2 = class test2 {
    constructor() {
        this.changed = false;
        this.last = Time.elapsed;
        this.last_dest = 1.0;
        this.mesh = null;
        this.texture = null;
        this.some = new Math.Vector3(0.0, 0.0, 0.0);
        this.someLol = new Math.Vector2(0.0, 1.0);
    }

    dotto() {
        console.log("success");
    }

    update() {
        lol();
        let posit = this.go.position;
        //console.log(posit.x, " ", posit.y, " ", posit.z);
        this.some.x = Math.Sin(Time.elapsed);
        World.setCameraPosition(this.some);


        let pos = new Math.Vector3(0.1, 0.2, 0.3);
        pos.x = 12.0;
        //console.log("test: ", pos.x, " ", pos.y, " ", pos.z)
        let pos2 = this.go.globalPosition;
        //console.log("test: ", pos2.x, " ", pos2.y, " ", pos2.z)

        if (!this.mesh) {
            this.mesh = Resources.getMesh("cow1");
        }

        if (!this.texture) {
            this.texture = Resources.getTexture("tex1.png");
        }
        if (this.changed === false && Input.isKeyboardKeyPressed("A")) {
            this.go.name = "heh";
            this.changed = true;
        }
        if (Time.elapsed > this.last + 1.0) {
            this.last = Time.elapsed;
            let n = World.createGameObject();
            const pos = new Math.Vector3(this.last, Math.Sin(this.last), 5.0);
            n.position = pos;
            n.name = "heh2";
            n.createComponent(Renderable);
            let renderable = n.getComponent(Renderable);
            renderable.mesh = this.mesh;
            renderable.texture = this.texture;
            //n.setScript("test_script");


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
