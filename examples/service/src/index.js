export function greet(who) {
  if (!who) throw new TypeError("greet needs somebody to greet");
  return "hello, " + who;
}
