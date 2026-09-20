# Stellarpool — güncel Testnet ürün planı

**Durum (20 Eylül 2026):** API v12 Testnet'te `CB5O6WCGCSA5PKH5HXWKO3WFEDHNEKJ5ZNW6EMNSERYDK6J4VYKLG46O` adresinde yayımlandı. Önceki v11 havuzları eski kontratta kalır. Mainnet tanıtımı işlemsizdir. Gerçek TL, gerçek ev/araç teslimi veya teslimat garantisi yoktur.

## Katılım

1. Kullanıcı **Ev, Araba veya Diğer** seçer; toplam bedeli, peşinat oranını ve ödeyebileceği aylık taksidi girer. Hızlı demo ayrı bir seçimdir ve dakikalık turlarla çalışır.
2. Peşinat düşüldükten sonra havuz hedefi bulunur. Gerekli kişi sayısı ve vade, `ceil(hedef / istenen aylık taksit)` ile hesaplanır; sözleşme 2–30 üyeye izin verir. Gerçek taksit, kuruş hassasiyetinde `hedef / kişi sayısı` üzerinden aşağı yuvarlanır ve ekranda gösterilir.
3. Aynı katkı, peşinat, teslim yöntemi ve takvim koşullarına sahip açık havuz varsa kullanıcı ona katılır. Yoksa havuz açılır ve kullanıcı katılır.
4. Ev ve Araba örnekleri kura, Diğer ve hızlı demo katılım sırası kullanır. Kura tüm üyeler o turun katkısını yatırınca çekilir. Rastgelelik hackathon düzeyindedir.
5. Normal planda katkı süreleri aylıktır. **Takvim otomatik hesaplanır; cüzdandan otomatik tahsilat yoktur.** Her üye her ay kendi ödemesini imzalar. Süre aşımını ya da ödeme adımını zincirde ilerletmek için birinin işlem çağırması gerekir.

Peşinat her üye tarafından katılırken kontrata yatırılır; üyenin kendi sırası gelince satıcıya giden alım tutarına eklenir. İptalde harcanmamış peşinat iade edilebilir. Peşinat ücret veya teslimat teminatı değildir.

## Havuz ve tur

1. Havuz dolunca sabit teslim sırası katılım sırasından **otomatik** kaydedilir; kura havuzunda önceden sıra yoktur. Kurucu ayrıca sıra veya doğrulayıcı adresi önermez.
2. Her üye aynı koşul sürümünü cüzdanıyla onaylar. Hepsi onaylayınca herkes havuzu başlatabilir. Başlangıçtan sonra üye listesi ve ekonomik koşullar kilitlidir.
3. Her turda bütün üyeler kendi katkısını yatırır. Eksik katkıda ek süre başlar; hâlâ eksikse tur durdurulur ve yalnızca bu turun katkıları iade edilir.
4. Bütün katkılar geldikten sonra sabit sıradaki alıcı veya kura kazananı havuzda kayıtlı **demo satıcısını** ve alım belgesinin SHA-256 özetini kaydeder. **Doğrulayıcı rolü, doğrulayıcı onayı ve alım için ikinci onay aşaması yoktur.** Alım süresi dolmadan herkes `execute_round` çağırabilir; tutar yalnızca kayıtlı satıcıya gider.
5. Alım süresi dolarsa yeni alım/ödeme yapılamaz. Havuz durdurulup henüz satıcıya gönderilmemiş turun katkıları iade edilir. Satıcıya ödenmiş önceki turlar geri alınamaz.

Kontrat fonu serbestçe kurucuya, platforma veya üyeye göndermez. Aynı varlığı kullanan havuzların yükümlülükleri ayrı tutulur. Transfer ve tur durum değişimi atomiktir.

## Açık risk ve kapsam

Erken teslim alan üye sonraki taksitleri ödemezse önceki turlarda diğer üyelerin ödediği tutar kontrattan geri alınamaz. Kura veya büyük grup bunu çözmez. Gerçek satıcı kimliği, tapu/ruhsat, rehin/ipotek ve tahsilat bu Testnet demosunda doğrulanmaz veya sunulmaz. Gerçek müşteri verisi zincire yazılmamalıdır. [Hukuki sınır](altin-gunu-legal-boundary.md) ve [lisanslı ürün yolu](legal-ai-path.md) ayrıca belgelenmiştir.

## Teknik karşılık

- `create_pool`, `join_pool`: sabit ekonomik koşulları kaydeder; son katılımda koşul sürümü 1 olur.
- `approve_terms`, `start_pool`, `cancel_unstarted_pool`: üye onayı, başlangıç ve kurulamayan havuzun iptali.
- `deposit`, `cure_payment`, `draw_recipient`: tur katkısı, ek sürede ödeme ve kurayla alıcı seçimi.
- `propose_purchase`, `execute_round`: alıcının alım kaydı ve kayıtlı demo satıcısına ödeme. Alım için doğrulayıcı eşiği yoktur.
- `mark_overdue`, `abort_pool`, `claim_refund`: süre aşımı ve mevcut tur iadesi.
- `get_pool`, `get_round`, `get_member_status`: zincirdeki durumun okunması.

API v12 için 33 sözleşme testi geçer; buna doğrulayıcısız alım ödeme akışı, otomatik katılım sırası, kura ve alım süresi bitiminde ödeme engeli dahildir. V12 Testnet dağıtıldı; gerçek Freighter cüzdanıyla uçtan uca işlem hâlâ gereklidir. Yayın ve eski havuz kanıtları [README](../README.md) içinde tutulur.
