"""Run with: python3 -m unittest -v"""
import unittest

from orders import CATALOG, LineItem, Product, line_total, money, order_total


class OrderTests(unittest.TestCase):
    def test_demo_total(self):
        items = [LineItem(CATALOG["notebook"], 2), LineItem(CATALOG["pen"], 3)]
        self.assertEqual(order_total(items, 20), 1880)
        self.assertEqual(money(1880), "$18.80")

    def test_half_cent_rounds_up(self):
        self.assertEqual(line_total(LineItem(Product("Sample", 101), 1), 50), 51)

    def test_empty_order(self):
        self.assertEqual(order_total([]), 0)

    def test_invalid_inputs(self):
        for quantity, discount in [(0, 0), (-1, 0), (1, -1), (1, 101)]:
            with self.subTest(quantity=quantity, discount=discount):
                with self.assertRaises(ValueError):
                    line_total(LineItem(CATALOG["pen"], quantity), discount)


if __name__ == "__main__":
    unittest.main()
