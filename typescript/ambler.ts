export type Nextable<S> = (state: S) => Next<any> | null;

export class Next<S> {
    constructor(private nextFunc: Nextable<S>, private state: S) {}

    run(): Next<any> | null {
        return this.nextFunc(this.state);
    }
}

export function amble<S>(initial: Nextable<S>, state: S): void {
    let next: Next<any> | null = initial(state);
    while (next) {
        next = next.run();
    }
}
