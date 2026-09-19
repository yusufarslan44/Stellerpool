/**
 * docs/plan.md bölüm 3: başlangıçta kilitlenmesi gereken sponsor güvencesinin alt sınırı
 * floor(N² / 4) × C. (N üye, C tur başına katkı). Kontratın kendi hesabı esastır; bu değer
 * yalnızca formda ve arayüzde bilgi vermek içindir.
 */
export function requiredGuarantee(memberCount: number, contribution: bigint): bigint {
  const n = BigInt(memberCount)
  return ((n * n) / 4n) * contribution
}
