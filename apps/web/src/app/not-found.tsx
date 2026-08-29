import Link from "next/link";

export default function NotFound() {
  return (
    <main className="wrap notfound">
      <p className="micro">404</p>
      <h1>这套皮肤不存在</h1>
      <p>
        <Link href="/" className="back">
          ← 回到主题索引
        </Link>
      </p>
    </main>
  );
}
