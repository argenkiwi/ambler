export class Next {
    constructor(nextFunc, state) {
        this.nextFunc = nextFunc;
        this.state = state;
    }

    run() {
        return this.nextFunc(this.state);
    }
}

export function amble(initial) {
    let next = initial;
    while (next) {
        next = next.run();
    }
}
