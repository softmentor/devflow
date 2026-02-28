import pkg from "./package.json" with { type: "json" };
export function hello() {
    return `Hello from ${pkg.name}`;
}
