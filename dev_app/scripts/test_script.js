class nice {
    constructor() {
        this.some = new Math.Vector();
        this.mesh = null;
    }

    update() {
//        console.log("wut from script");

        this.some.x = Math.Sin(Time.elapsed);
        this.go.position = this.some;

        let pos = this.go.position;
//        console.log("x: ", pos.x, ", y: ", pos.y, ", z: ", pos.z);
        this.go.test();

        const objs = World.findByName("heh");

        console.log("Jest ", objs.length, " obiektow");
        
        console.log("pozycja myszki: ", Input.mousePosition().x, " ", Input.mousePosition().y);

        if (Input.isKeyboardKeyPressed("A")) {
            this.some.z += 0.1;
        }
        if (Input.isKeyboardKeyPressed("D")) {
            this.some.z -= 0.1;
        }

        if (Input.isKeyboardKeyPressed("W")) {
            this.some.y += 0.1;
        }
        if (Input.isKeyboardKeyPressed("S")) {
            this.some.y -= 0.1;
        }
    }
}
