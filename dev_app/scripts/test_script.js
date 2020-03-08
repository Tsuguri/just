class nice {
    constructor() {
        this.some = new Math.Vector();
        this.mesh = null;
    }

    update() {
        //        console.log("wut from script");


        let n = World.findByName("super_obj");
        if (n.length > 0) {
            let go = n[0];
            let script = go.getScript();
            if (script !== null) {
                script.dotto();
            }
        }
    }
}
