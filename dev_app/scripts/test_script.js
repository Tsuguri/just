class nice {
    constructor() {
        this.opt1 = 10;
        this.opt2 = 12;
        this.opt3 = new Math.Vector(1,2,3);
        this.go = {};
        //console.log("not workuuuuuuuuuuuuuuuuuuung");
    }

    update() {
        console.log("wut from script");
        const wut =this.go.get_position();
        this.go.test();
    }
}
