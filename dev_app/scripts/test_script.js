class nice {
    constructor() {
        this.some = new Math.Vector();
    }

    update() {
        console.log("wut from script");
        let tmp = this.some.clone();

        tmp.x = Math.Sin(Time.elapsed);
        this.go.set_position(new Math.Vector(Math.Sin(Time.elapsed), 2.0, 13.0));

        let pos = this.go.get_position();
        console.log("x: ", pos.x, ", y: ", pos.y, ", z: ", pos.z);
        this.go.test();
    }
}
