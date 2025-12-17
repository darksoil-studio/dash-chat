import { fromByteArray, toByteArray } from 'base64-js';
// @ts-ignore
import { decode, encode } from 'cbor-web';

import { ContactCode } from '../types';

export function encodeContactCode(contactCode: ContactCode): string {
	const bin = encode([
		contactCode.member_code,
		contactCode.inbox_topic,
		contactCode.device_space_id,
		contactCode.chat_actor_id,
		contactCode.share_intent,
	]);
	return fromByteArray(bin);
}

export function decodeContactCode(contactCodeString: string): ContactCode {
	const bin = toByteArray(contactCodeString);
	const [
		member_code,
		inbox_topic,
		device_space_id,
		chat_actor_id,
		share_intent,
	] = decode(bin);
	return {
		member_code,
		inbox_topic,
		device_space_id,
		chat_actor_id,
		share_intent,
	};
}


export const compress = async (
  str: string,
  encoding = 'gzip' as CompressionFormat
): Promise<ArrayBuffer> => {
  const byteArray = new TextEncoder().encode(str)
  const cs = new CompressionStream(encoding)
  const writer = cs.writable.getWriter()
  writer.write(byteArray)
  writer.close()
  return new Response(cs.readable).arrayBuffer()
}

export const decompress = async (
  byteArray: ArrayBuffer,
  encoding = 'gzip' as CompressionFormat
): Promise<string> => {
  const cs = new DecompressionStream(encoding)
  const writer = cs.writable.getWriter()
  writer.write(byteArray)
  writer.close()
  const arrayBuffer = await new Response(cs.readable).arrayBuffer()
  return new TextDecoder().decode(arrayBuffer)
}
