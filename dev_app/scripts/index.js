class test2 {
    constructor() {
        this.changed = false;
        this.last = Time.elapsed();
        this.last_dest = 1.0;
        this.mesh = null;
        this.texture = null;
        this.some = new Math.Vector3(0.0, 0.0, 0.0);
        this.someLol = new Math.Vector2(0.0, 1.0);
        this.create = true;
    }

    update() {
        let posit = this.go.position;
        this.some.x = Math.Sin(Time.elapsed());
        World.setCameraPosition(this.some); // to jeszcze nie działa :)


        let pos = new Math.Vector3(0.1, 0.2, 0.3);
        pos.x = 12.0;
        let pos2 = this.go.globalPosition;

        // if (!this.mesh) {
        //     this.mesh = Resources.getMesh("cow1");
        // }

        // if (!this.texture) {
        //     this.texture = Resources.getTexture("tex1.png");
        // }
        if (Input.keyPressedInLastFrame("A")) {
            console.log("changing state :)")
            this.create = !this.create;
        }
        if (this.create && Time.elapsed() > this.last + 1.0) {
            console.log("creating new cow");
            this.last = Time.elapsed();
            let n = World.createGameObject();

            World.aniasFunction({
                name: "heh2",
                position: new Math.Vector3(this.last, Math.Sin(this.last), 5.0),
                mesh: "cow1"
            });
        }
        if (this.create && Time.elapsed() > this.last_dest + 2.0) {
            this.last_dest = Time.elapsed();
            console.log("destroying");
            let n = World.findByName("heh2");
            if (n.length > 0) {
                n[0].destroy();
            }
        }
    }
}


console.log("initializing world");

let floor = World.aniasFunction({
    name: "floor",
    mesh: "floor",
    position: new Math.Vector3(-20, -2, 20),
    scale: new Math.Vector3(10, 10, 10)
});

let position = new Math.Vector3(10, 20, 30);
let go2 = World.aniasFunction({
    name: "heheszko", position: position, controller: new test2()
});

console.log("position: ", position.x, " ", position.y, " ", position.z);