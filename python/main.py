import asyncio

from ambler import amble
from lead import Lead
from step.count import count
from step.start import start
from step.stop import stop


async def main():
    initial_state = 0
    initial_lead = Lead.START

    def follow(lead: Lead):
        if lead == Lead.START:
            return start
        elif lead == Lead.COUNT:
            return count
        elif lead == Lead.STOP:
            return stop
        else:
            raise ValueError(f"Unknown lead: {lead}")

    await amble(initial_state, initial_lead, follow)


if __name__ == "__main__":
    asyncio.run(main())
