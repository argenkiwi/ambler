export class Next {
    constructor(public run: () => Promise<Next | null>) {}
}

export async function amble(initial: Next | null): Promise<void> {
    let next = initial;
    while (next) {
        next = await next.run();
    }
}

export async function ambleFrom(initial: () => Promise<Next | null>): Promise<void> {
    await amble(new Next(initial));
}
