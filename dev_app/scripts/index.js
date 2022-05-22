console.log("initializing world");

let go = World.createGameObject();
let position = new Math.Vector3(10, 20, 30);
let go2 = World.aniasFunction({ name: "heheszko", position: position });

console.log("position: ", position.x, " ", position.y, " ", position.z);