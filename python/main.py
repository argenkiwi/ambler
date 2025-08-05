import asyncio

from ambler import amble
from lead import Lead
from step.count import count
from step.start import start
from step.stop import stop


async def main():
    initial_state = 0
    initial_lead = Lead.START

    async def follow(lead: Lead, state: int):
        if lead == Lead.START:
            return start(state)
        elif lead == Lead.COUNT:
            return await count(state)
        elif lead == Lead.STOP:
            return stop(state)
        else:
            raise ValueError(f"Unknown lead: {lead}")

    await amble(initial_state, initial_lead, follow)


if __name__ == "__main__":
    asyncio.run(main())
