/** A product whose fields are offered by member completion. */
export interface Product {
  readonly name: string;
  price: number;
  category: "book" | "tool";
}

/** Apply a fractional discount, such as 0.2 for twenty percent. */
export function discountedPrice(product: Product, discount = 0.1): number {
  return Math.round(product.price * (1 - discount) * 100) / 100;
}

export const catalog: Product[] = [
  { name: "Literate programming", price: 30, category: "book" },
  { name: "Notebook", price: 8, category: "tool" },
];
