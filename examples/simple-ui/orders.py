"""A dependency-free order calculator for the simple UI playground."""
from __future__ import annotations

import argparse
from dataclasses import dataclass
import time


@dataclass(frozen=True)
class Product:
    """A catalog item with an integer price in cents."""

    name: str
    price_cents: int


@dataclass(frozen=True)
class LineItem:
    product: Product
    quantity: int


CATALOG = {
    "notebook": Product("Notebook", 800),
    "pen": Product("Pen", 250),
    "guide": Product("Field guide", 2400),
}


def line_total(item: LineItem, discount_percent: int = 0) -> int:
    """Return discounted cents, rounding half a cent upward."""
    if item.quantity < 1:
        raise ValueError("quantity must be positive")
    if item.product.price_cents < 0:
        raise ValueError("price cannot be negative")
    if not 0 <= discount_percent <= 100:
        raise ValueError("discount must be between 0 and 100")
    subtotal = item.product.price_cents * item.quantity
    discounted = (subtotal * (100 - discount_percent) + 50) // 100
    return discounted


def order_total(items: list[LineItem], discount_percent: int = 0) -> int:
    """Sum line totals; put a breakpoint on the line_total call."""
    total = 0
    for item in items:
        amount = line_total(item, discount_percent)
        total += amount
    return total


def money(cents: int) -> str:
    return f"${cents // 100}.{cents % 100:02d}"


def demo() -> int:
    items = [LineItem(CATALOG["notebook"], 2), LineItem(CATALOG["pen"], 3)]
    total = order_total(items, discount_percent=20)
    print(f"Order: {money(total)}")
    print(f"Items: {sum(item.quantity for item in items)}")
    return total


# Language probes (undo each probe after trying it):
# Completion: type CATALOG["notebook"]. on a new line, then Ctrl-Space.
# Hover: inspect Product, LineItem or line_total.
# Signature: type line_total( in a new expression.
# Diagnostic: uncomment the next line; quantity expects an integer.
# invalid = LineItem(CATALOG["pen"], "three")
# Definition in browser: Ctrl/Cmd-click the demo call below.


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--watch", action="store_true", help="repeat until Interrupt")
    arguments = parser.parse_args()
    try:
        if arguments.watch:
            while True:
                demo()
                time.sleep(1)
        else:
            demo()
    except KeyboardInterrupt:
        print("Stopped.")
