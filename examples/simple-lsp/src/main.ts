import { catalog, discountedPrice, type Product } from "./catalog.js";

// Hover Product or discountedPrice for type information and documentation.
const featured: Product = catalog[0];

// Completion: on a new line type featured. or featured.name.
// Explicit completion: press Ctrl-Space after typing featured.
// Signature help: on a new line type discountedPrice(
const total = discountedPrice(featured, 0.2);
console.log(`${featured.name}: $${total.toFixed(2)}`);

// Diagnostics: uncomment the next line, then remove it to recover.
// const invalid: Product = { name: "Broken", price: "free", category: "book" };

// Same-file definition: Ctrl/Cmd-click describe below to jump to its declaration.
function describe(product: Product) {
  return `${product.category}: ${product.name}`;
}
console.log(describe(featured));
