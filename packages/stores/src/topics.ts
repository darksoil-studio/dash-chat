import { blake3Hash } from '@webbuf/blake3';
import { WebBuf } from 'webbuf';

import { ActorId, PublicKey, TopicId } from './p2panda/types';

// const fromHexString = (hexString:string) =>
//   Uint8Array.from(hexString.match(/.{1,2}/g)!.map((byte) => parseInt(byte, 16)));

// export function personalTopicFor(publicKey: PublicKey): TopicId {
// const bytes =fromHexString(publicKey)
// const hash = blake3Hash( WebBuf.fromHex(publicKey))
// console.log(WebBuf.fromHex(publicKey),hash.buf)
// return `${hash.buf.toHex()}`;
// }
export function personalTopicFor(chatActorId: ActorId): TopicId {
	return chatActorId;
}
