export class Next {
    constructor(nextFunc, state) {
        this.nextFunc = nextFunc;
        this.state = state;
    }

    run() {
        return this.nextFunc(this.state);
    }
}

export async function amble(initial, state) {
    let next = await initial(state);
    while (next) {
        next = await next.run();
    }
}
