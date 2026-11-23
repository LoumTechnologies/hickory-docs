import { unitPriceCents } from "./catalog";

export function receiptTotalCents(quantity: number): number {
  return unitPriceCents * quantity;
}
