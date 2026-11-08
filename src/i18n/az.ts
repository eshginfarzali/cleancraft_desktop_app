import type { MessageKey } from './en';

/** Azərbaycan dili. */
export const az: Record<MessageKey, string> = {
  'app.tagline':
    '24 saatdır dəyişdirilməyən cache və müvəqqəti faylları silir. Yalnız Parametrlərdə aktiv etdiyin qovluqlara toxunulur.',
  'tab.clean': 'Təmizlə',
  'tab.history': 'Tarixçə',
  'tab.settings': 'Parametrlər',

  'scan.button': 'Skan et',
  'scan.again': 'Yenidən skan et',
  'scan.scanning': 'Skan edilir…',
  'scan.empty': 'Heç bir hədəf aktiv deyil. Parametrlərdən seç.',
  'scan.summary': '{count} fayldan {size}-a qədər yer boşalda bilərsən.',
  'scan.target.detail': '{count} fayl · {size}',
  'scan.target.unreadable': '{count} qovluq oxunmadı',
  'scan.preview': 'Fayllara bax',

  'clean.selected': 'Seçilənləri təmizlə ({size})',
  'clean.cleaning': 'Təmizlənir…',
  'confirm.wholeTargets':
    'Seçilmiş qovluqlardan {size} birdəfəlik silinsin? Bunu geri qaytarmaq olmur.',
  'confirm.delete': 'Sil',
  'confirm.cancel': 'Ləğv et',
  'result.freed': '{count} fayl silindi, {size} yer boşaldı.',
  'result.failed':
    '{count} fayl silinmədi (istifadədədir və ya qorunur) və yerində qaldı.',
  'result.rejected':
    'Seçilmiş {count} element dəyişdiyi və ya icazə verilmədiyi üçün buraxıldı.',

  'preview.title': 'Önizləmə: {target}',
  'preview.close': 'Bağla',
  'preview.selectAll': 'Hamısını seç',
  'preview.selectNone': 'Seçimi təmizlə',
  'preview.selected': '{total} elementdən {count} seçilib · {size}',
  'preview.truncated':
    'Ən böyük {shown} element göstərilir. Bütün hədəfi təmizləsən, {total} elementin hamısı silinir.',
  'preview.empty': 'Hazırda burada silinəcək bir şey yoxdur.',
  'preview.deleteSelected': 'Seçilənləri sil ({size})',
  'preview.confirm':
    'Seçilmiş {count} element ({size}) birdəfəlik silinsin? Bunu geri qaytarmaq olmur.',
  'preview.column.path': 'Yol',
  'preview.column.size': 'Ölçü',
  'preview.column.modified': 'Son dəyişiklik',
  'preview.folderOf': 'qovluq, {count} fayl',

  'history.empty': 'Hələ təmizləmə olmayıb.',
  'history.total': 'İndiyə qədər {count} təmizləmədə {size} boşaldılıb.',
  'history.clear': 'Tarixçəni sil',
  'history.clearConfirm':
    'Təmizləmə tarixçəsi silinsin? Silinmiş fayllara təsiri yoxdur.',
  'history.trigger.manual': 'Əl ilə',
  'history.trigger.selective': 'Seçilmiş fayllar',
  'history.trigger.scheduled': 'Planlı',
  'history.entry': '{size} · {count} fayl',
  'history.entryFailed': '{count} uğursuz',

  'settings.targets': 'Nə təmizlənsin',
  'settings.targets.help':
    'Hər qovluq tətbiqdə əvvəlcədən müəyyən edilib, öz yolunu əlavə edə bilməzsən. Yalnız 24 saatdan köhnə fayllar silinir.',
  'settings.unavailable': 'Bu kompüterdə tapılmadı',
  'settings.category.system': 'Sistem',
  'settings.category.browser': 'Brauzerlər',
  'settings.category.developer': 'Proqramçı alətləri',
  'settings.schedule': 'Həftəlik təmizləmə',
  'settings.schedule.enabled': 'Aktiv hədəfləri hər həftə avtomatik təmizlə',
  'settings.schedule.when': 'Gün və saat',
  'settings.schedule.help':
    'CleanCraft açıq olanda və ya system tray-də işləyəndə işə düşür. Həmin vaxt kompüter sönülüdürsə, CleanCraft növbəti dəfə işləyəndə təmizləyir.',
  'settings.appearance': 'Görünüş',
  'settings.theme': 'Tema',
  'settings.theme.system': 'Sistem',
  'settings.theme.light': 'İşıqlı',
  'settings.theme.dark': 'Qaranlıq',
  'settings.language': 'Dil',

  'weekday.0': 'Bazar ertəsi',
  'weekday.1': 'Çərşənbə axşamı',
  'weekday.2': 'Çərşənbə',
  'weekday.3': 'Cümə axşamı',
  'weekday.4': 'Cümə',
  'weekday.5': 'Şənbə',
  'weekday.6': 'Bazar',

  'target.temp': 'Müvəqqəti fayllar',
  'target.user-cache': 'Tətbiq cache-ləri',
  'target.chrome': 'Google Chrome cache',
  'target.edge': 'Microsoft Edge cache',
  'target.firefox': 'Firefox cache',
  'target.npm': 'npm cache',
  'target.yarn': 'Yarn cache',
  'target.pnpm': 'pnpm store',
  'target.xcode-derived-data': 'Xcode DerivedData (layihə üzrə)',

  'error.prefix': 'Xəta baş verdi: {message}',
};
