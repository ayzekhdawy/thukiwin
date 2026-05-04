/**
 * Gateway tab - local gateway server settings.
 */

import { Section, NumberStepper, Toggle } from '../components';
import { SaveField } from '../components/SaveField';
import { configHelp } from '../configHelpers';
import type { RawAppConfig } from '../types';

interface GatewayTabProps {
  config: RawAppConfig;
  resyncToken: number;
  onSaved: (next: RawAppConfig) => void;
}

export function GatewayTab({ config, resyncToken, onSaved }: GatewayTabProps) {
  return (
    <>
      <Section heading="Server">
        <SaveField
          section="gateway"
          fieldKey="enabled"
          label="Enable gateway"
          helper={configHelp('gateway', 'enabled')}
          initialValue={config.gateway.enabled}
          resyncToken={resyncToken}
          onSaved={onSaved}
          rightAlign
          render={(value, setValue) => (
            <Toggle checked={value} onChange={setValue} ariaLabel="Enable gateway" />
          )}
        />
        <SaveField
          section="gateway"
          fieldKey="port"
          label="Port"
          helper={configHelp('gateway', 'port')}
          initialValue={config.gateway.port}
          resyncToken={resyncToken}
          onSaved={onSaved}
          render={(value, setValue) => (
            <NumberStepper value={value} min={1024} max={65535} onChange={setValue} ariaLabel="Gateway port" />
          )}
        />
      </Section>
    </>
  );
}