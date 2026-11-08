import { useI18n } from '../i18n';

interface Props {
  message: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export default function Confirm({ message, onConfirm, onCancel }: Props) {
  const { t } = useI18n();
  return (
    <div className="confirm" role="alertdialog" aria-label={message}>
      <p>{message}</p>
      <div className="row">
        <button className="danger" onClick={onConfirm}>
          {t('confirm.delete')}
        </button>
        <button onClick={onCancel}>{t('confirm.cancel')}</button>
      </div>
    </div>
  );
}
