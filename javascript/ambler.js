export class Next {
    constructor(run) {
        this.run = run;
    }
}

export async function amble(initial) {
    let next = initial;
    while (next) {
        next = await next.run();
    }
}

export function ambleFrom(initial) {
    return amble(new Next(initial));
}
