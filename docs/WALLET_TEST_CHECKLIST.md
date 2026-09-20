# API v12 — gerçek cüzdanla Testnet kontrol listesi

Bu liste, **yeni v12 kontratı dağıtılıp site ona bağlandıktan sonra** uygulanır. Yayındaki v11 havuzları eski akışı kullanır; v12 testine örnek sayılmaz. Gerçek para veya mülkiyet aktarımı yoktur.

## Hazırlık

- [ ] Site yapılandırmasındaki sözleşmede `version()` değerinin 12 olduğunu doğrula.
- [ ] Freighter'ı Testnet'e ayarla; seçilen üye sayısı kadar ayrı test hesabı hazırla. **Doğrulayıcı hesabı gerekmez.**
- [ ] Her üyeye Testnet XLM ve havuzun kullandığı test varlığından, tüm katkı ve peşinatlara yetecek bakiye sağla.
- [ ] Kısa deneme için katılım ekranındaki **Hızlı demo** seçeneğini kullan. Ev, Araba ve Diğer normal planları 30 günlük katkı süresiyle kurulur.

## Temel akış

- [ ] Ev, Araba veya Diğer seç; bedel, peşinat ve ödeyebileceğin aylık taksidi gir. Hesaplanan taksit, kişi sayısı ve vadeyi not et.
- [ ] Uygun açık havuz varsa ona katıl; yoksa oluşturma ve katılma işlemlerini cüzdanda imzala. Her iki işlem hash'ini kaydet.
- [ ] Diğer üyeler katılınca sabit sırada katılım sırasının otomatik oluştuğunu; kurada önceden sıra olmadığını doğrula.
- [ ] Bütün üyeler koşul sürümü 1'i onaylasın; herhangi biri havuzu başlatsın.
- [ ] Her üye kendi tur katkısını cüzdanıyla yatırıp imzalasın. Normal planda sonraki ödeme de otomatik tahsil edilmez; üye imzası gerekir.
- [ ] Kura havuzunda tüm katkılar gelince kura çek; kazananın daha önce almış biri olmadığını doğrula.
- [ ] Alıcı kayıtlı demo satıcısını ve alım belgesi özetini kaydetsin. **Doğrulayıcı onayı adımı olmadan** `execute_round` çağır; satıcı bakiyesini ve tur durumunu doğrula.
- [ ] Son tura kadar sürdür; havuzun `Completed` olduğunu, satıcıya giden toplamı ve sözleşme bakiyesini kontrol et.

## Gecikme ve iade

- [ ] Ayrı hızlı demo havuzunda bir üyenin katkısını atla. Son tarih geçince `mark_overdue`, ek süre bitince `abort_pool` çağır.
- [ ] Yalnızca mevcut turda katkı yapmış üyenin katkısını ve harcanmamış peşinatları iade al; önceki ödenmiş turların geri gelmediğini doğrula.
- [ ] İkinci iade denemesinin reddedildiğini kontrol et.

## Kanıt

Cüzdan imzaları, işlem hash'leri ve Stellar ağ gezgini bağlantılarını kaydet. Test anchor'ından yükleme varsa SEP-10/24 akışını ayrıca doğrula; Testnet varlığı gerçek TRY değildir. Sorun çıktığında ilgili işlem hash'i, havuz numarası, adım ve hata mesajını sakla.
