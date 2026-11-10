import {DatabaseSync} from 'node:sqlite';
import {readFileSync} from 'node:fs';
export function database(){
 const db=new DatabaseSync(':memory:');db.exec(readFileSync(new URL('../drizzle/0000_presence.sql',import.meta.url),'utf8'));
 return {raw:db,prepare(sql){let args=[];return {bind(...values){args=values;return this;},async run(){return db.prepare(sql).run(...args);},async first(){return db.prepare(sql).get(...args)||null;},async all(){return {results:db.prepare(sql).all(...args)};}};},async batch(statements){const result=[];for(const statement of statements)result.push(await statement.run());return result;}};
}
