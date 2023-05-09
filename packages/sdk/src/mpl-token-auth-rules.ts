import {
  RuleSetHeader,
  ruleSetHeaderBeet,
  RuleSetRevisionMapV1,
  ruleSetRevisionMapV1Beet,
} from './generated';

import type { bignum } from '@metaplex-foundation/beet';
import { decode } from '@msgpack/msgpack';
import { BN } from 'bn.js';
import { deserializeRuleSetV2, RuleSetV2 } from './ruleSetV2';
import { RuleSetV1 } from './ruleSetV1';

export * from './errors';
export * from './generated';
export * from './pda';
export * from './ruleSetV1';
export * from './ruleSetV2';
export * from './shared';

export const PREFIX = 'rule_set';

export const getHeader = (data: Buffer): RuleSetHeader => {
  const [header] = ruleSetHeaderBeet.deserialize(data.subarray(0, 9));
  return header;
};

export const getRevisionMapV1 = (data: Buffer): RuleSetRevisionMapV1 => {
  const header = getHeader(data);
  const [revmap] = ruleSetRevisionMapV1Beet.deserialize(
    data.subarray(bignumToNumber(header.revMapVersionLocation) + 1, data.length),
  );
  return revmap;
};

export const getLatestRuleSet = (data: Buffer): RuleSetV1 | RuleSetV2 => {
  const header = getHeader(data);
  const revmap = getRevisionMapV1(data);
  const latestRevision = bignumToNumber(
    revmap.ruleSetRevisions[revmap.ruleSetRevisions.length - 1],
  );
  const ruleSetVersion = data[latestRevision];
  const endOfRuleSet = bignumToNumber(header.revMapVersionLocation);
  switch (ruleSetVersion) {
    case 1:
      return decode(data.subarray(latestRevision + 1, endOfRuleSet)) as RuleSetV1;
    case 2:
      return deserializeRuleSetV2(data.subarray(latestRevision, endOfRuleSet));
    default:
      throw new Error('Unknown ruleset version: ' + ruleSetVersion);
  }
};

function bignumToNumber(bignum: bignum): number {
  return new BN(bignum).toNumber();
}
